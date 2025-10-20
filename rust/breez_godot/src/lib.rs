use godot::prelude::*;
use breez_sdk_spark::{
    connect, default_config, ConnectRequest, Network, Seed, BreezSdk,
    GetInfoRequest, ReceivePaymentRequest, ReceivePaymentMethod,
    PrepareSendPaymentRequest, SendPaymentRequest, SendPaymentOptions,
    ListPaymentsRequest, SyncWalletRequest, ListUnclaimedDepositsRequest,
    ClaimDepositRequest, Fee, RegisterLightningAddressRequest,
    CheckLightningAddressRequest,
};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

struct BreezExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BreezExtension {}

/// Godot class for Breez Spark SDK integration
#[derive(GodotClass)]
#[class(base=Node)]
pub struct BreezNode {
    #[base]
    base: Base<Node>,
    sdk: Arc<Mutex<Option<BreezSdk>>>,
    runtime: Arc<Runtime>,
}

#[godot_api]
impl INode for BreezNode {
    fn init(base: Base<Node>) -> Self {
        godot_print!("BreezNode initialized");
        Self {
            base,
            sdk: Arc::new(Mutex::new(None)),
            runtime: Arc::new(Runtime::new().expect("Failed to create tokio runtime")),
        }
    }
}

#[godot_api]
impl BreezNode {
    /// Connect to Breez SDK
    /// 
    /// # Arguments
    /// * `mnemonic` - 12 or 24 word BIP39 mnemonic phrase
    /// * `api_key` - Your Breez API key
    /// * `network` - "mainnet" or "regtest"
    /// * `storage_dir` - Directory to store wallet data
    #[func]
    pub fn connect_sdk(
        &mut self,
        mnemonic: GString,
        api_key: GString,
        network: GString,
        storage_dir: GString,
    ) -> bool {
        godot_print!("Connecting to Breez Spark SDK...");
        
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let mnemonic_str = mnemonic.to_string();
        let api_key_str = api_key.to_string();
        let network_str = network.to_string();
        let storage_dir_str = storage_dir.to_string();
        
        let result: Result<(), Box<dyn std::error::Error>> = runtime.block_on(async move {
            let seed = Seed::Mnemonic {
                mnemonic: mnemonic_str,
                passphrase: None,
            };

            let network_type = match network_str.as_str() {
                "mainnet" => Network::Mainnet,
                "regtest" => Network::Regtest,
                _ => {
                    godot_error!("Invalid network: {}", network_str);
                    return Err("Invalid network".into());
                }
            };

            let mut config = default_config(network_type);
            config.api_key = Some(api_key_str);

            match connect(ConnectRequest {
                config,
                seed,
                storage_dir: storage_dir_str,
            }).await {
                Ok(sdk) => {
                    *sdk_arc.lock().unwrap() = Some(sdk);
                    godot_print!("✅ Connected to Breez Spark SDK");
                    Ok(())
                }
                Err(e) => {
                    godot_error!("Failed to connect: {:?}", e);
                    Err(format!("Failed to connect: {:?}", e).into())
                }
            }
        });

        result.is_ok()
    }

    /// Get wallet balance in satoshis
    #[func]
    pub fn get_balance(&self) -> i64 {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let result: Result<i64, Box<dyn std::error::Error>> = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                match sdk.get_info(GetInfoRequest {
                    ensure_synced: Some(true),
                }).await {
                    Ok(info) => Ok(info.balance_sats as i64),
                    Err(e) => {
                        godot_error!("Failed to get balance: {:?}", e);
                        Ok(0)
                    }
                }
            } else {
                godot_warn!("SDK not initialized");
                Ok(0)
            }
        });

        result.unwrap_or(0)
    }

    /// Get a Bitcoin address for receiving on-chain funds
    #[func]
    pub fn get_bitcoin_address(&self) -> GString {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                match sdk.receive_payment(ReceivePaymentRequest {
                    payment_method: ReceivePaymentMethod::BitcoinAddress,
                }).await {
                    Ok(response) => Ok(response.payment_request),
                    Err(e) => Err(format!("Failed to get address: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });

        match result {
            Ok(address) => GString::from(&address),  // Use &String instead of String
            Err(e) => {
                godot_error!("{}", e);
                GString::from("")
            }
        }
    }

    /// Create a Lightning invoice
    /// 
    /// # Arguments
    /// * `amount_sats` - Amount in satoshis (0 for any amount)
    /// * `description` - Invoice description
    #[func]
    pub fn create_invoice(&self, amount_sats: i64, description: GString) -> GString {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        let desc = description.to_string();
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                let amount = if amount_sats > 0 {
                    Some(amount_sats as u64)
                } else {
                    None
                };

                match sdk.receive_payment(ReceivePaymentRequest {
                    payment_method: ReceivePaymentMethod::Bolt11Invoice {
                        description: desc,
                        amount_sats: amount,
                    },
                }).await {
                    Ok(response) => Ok(response.payment_request),
                    Err(e) => Err(format!("Failed to create invoice: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });

        match result {
            Ok(invoice) => {
                godot_print!("✅ Invoice created");
                GString::from(&invoice)  // Use &String instead of String
            }
            Err(e) => {
                godot_error!("{}", e);
                GString::from("")
            }
        }
    }

    /// Pay a Lightning invoice (two-step process: prepare then send)
    /// 
    /// # Arguments
    /// * `bolt11` - The BOLT11 invoice string
    /// * `timeout_secs` - Timeout in seconds for payment completion (0 for default)
    #[func]
    pub fn pay_invoice(&self, bolt11: GString, timeout_secs: i64) -> Dictionary {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        let invoice = bolt11.to_string();
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                // Step 1: Prepare the payment
                let prepare_response = match sdk.prepare_send_payment(PrepareSendPaymentRequest {
                    payment_request: invoice.clone(),
                    amount_sats: None,  // Only needed for amountless invoices
                }).await {
                    Ok(response) => response,
                    Err(e) => return Err(format!("Failed to prepare payment: {:?}", e)),
                };

                // Step 2: Send the payment with optional timeout
                let options = if timeout_secs > 0 {
                    Some(SendPaymentOptions::Bolt11Invoice {
                        prefer_spark: false,  // Can be set to true to prefer Spark transfer
                        completion_timeout_secs: Some(timeout_secs as u32),
                    })
        } else {
                    None
                };

                match sdk.send_payment(SendPaymentRequest {
                    prepare_response,
                    options,
                }).await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(format!("Payment failed: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });

        let mut dict = Dictionary::new();
        
        match result {
            Ok(payment) => {
                godot_print!("✅ Payment sent");
                dict.set("success", true);
                dict.set("payment_id", payment.payment.id);
                dict.set("amount", payment.payment.amount as i64);
            }
            Err(e) => {
                godot_error!("{}", e);
                dict.set("success", false);
                dict.set("error", e);
            }
        }
        
        dict
    }

    /// Get Spark address for receiving payments
    #[func]
    pub fn get_spark_address(&self) -> GString {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                match sdk.receive_payment(ReceivePaymentRequest {
                    payment_method: ReceivePaymentMethod::SparkAddress,
                }).await {
                    Ok(response) => Ok(response.payment_request),
                    Err(e) => Err(format!("Failed to get Spark address: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });

        match result {
            Ok(address) => GString::from(&address),  // Use &String instead of String
            Err(e) => {
                godot_error!("{}", e);
                GString::from("")
            }
        }
    }

    /// Check if SDK is connected
    #[func]
    pub fn is_sdk_connected(&self) -> bool {
        self.sdk.lock().unwrap().is_some()
    }

    /// Disconnect from SDK
    #[func]
    pub fn disconnect_breez(&mut self) {
        let mut sdk_guard = self.sdk.lock().unwrap();
        if sdk_guard.is_some() {
            *sdk_guard = None;
            godot_print!("Disconnected from Breez SDK");
        }
    }

    /// Manually sync the wallet
    #[func]
    pub fn sync_wallet(&self) -> bool {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                match sdk.sync_wallet(SyncWalletRequest {}).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Failed to sync: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });
        
        match result {
            Ok(_) => {
                godot_print!("✅ Wallet synced");
                true
            }
            Err(e) => {
                godot_error!("{}", e);
                false
            }
        }
    }

    /// List payment history
    /// 
    /// # Arguments
    /// * `offset` - Number of payments to skip (for pagination)
    /// * `limit` - Maximum number of payments to return
    #[func]
    pub fn list_payments(&self, offset: i64, limit: i64) -> Array<Dictionary> {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                match sdk.list_payments(ListPaymentsRequest {
                    offset: if offset > 0 { Some(offset as u32) } else { None },
                    limit: if limit > 0 { Some(limit as u32) } else { None },
                }).await {
                    Ok(response) => Ok(response.payments),
                    Err(e) => Err(format!("Failed to list payments: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });
        
        let mut array = Array::new();
        
        match result {
            Ok(payments) => {
                for payment in payments {
                    let mut dict = Dictionary::new();
                    dict.set("id", payment.id);
                    dict.set("amount", payment.amount as i64);
                    dict.set("fees", payment.fees as i64);
                    dict.set("timestamp", payment.timestamp as i64);
                    dict.set("status", payment.status.to_string());
                    dict.set("payment_type", payment.payment_type.to_string());
                    dict.set("method", payment.method.to_string());
                    array.push(&dict);
                }
            }
            Err(e) => {
                godot_error!("{}", e);
            }
        }
        
        array
    }

    /// List unclaimed deposits
    #[func]
    pub fn list_unclaimed_deposits(&self) -> Array<Dictionary> {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                match sdk.list_unclaimed_deposits(ListUnclaimedDepositsRequest {}).await {
                    Ok(response) => Ok(response.deposits),
                    Err(e) => Err(format!("Failed to list deposits: {:?}", e)),
            }
        } else {
                Err("SDK not initialized".to_string())
            }
        });
        
        let mut array = Array::new();
        
        match result {
            Ok(deposits) => {
                for deposit in deposits {
                    let mut dict = Dictionary::new();
                    dict.set("txid", deposit.txid);
                    dict.set("vout", deposit.vout);
                    dict.set("amount_sats", deposit.amount_sats as i64);
                    array.push(&dict);
                }
            }
            Err(e) => {
                godot_error!("{}", e);
            }
        }
        
        array
    }

    /// Claim a specific deposit
    /// 
    /// # Arguments
    /// * `txid` - Transaction ID
    /// * `vout` - Output index
    /// * `max_fee_sats` - Maximum fee to pay for claiming (0 for any fee)
    #[func]
    pub fn claim_deposit(&self, txid: GString, vout: i64, max_fee_sats: i64) -> Dictionary {
        let sdk_arc = Arc::clone(&self.sdk);
        let runtime = Arc::clone(&self.runtime);
        let txid_str = txid.to_string();
        
        let result = runtime.block_on(async move {
            let sdk_guard = sdk_arc.lock().unwrap();
            if let Some(sdk) = sdk_guard.as_ref() {
                let max_fee = if max_fee_sats > 0 {
                    Some(Fee::Fixed { amount: max_fee_sats as u64 })
                } else {
                    None
                };
                
                match sdk.claim_deposit(ClaimDepositRequest {
                    txid: txid_str,
                    vout: vout as u32,
                    max_fee,
                }).await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(format!("Failed to claim deposit: {:?}", e)),
                }
            } else {
                Err("SDK not initialized".to_string())
            }
        });
        
        let mut dict = Dictionary::new();
        
        match result {
            Ok(response) => {
                godot_print!("✅ Deposit claimed");
                dict.set("success", true);
                dict.set("payment_id", response.payment.id);
            }
            Err(e) => {
                godot_error!("{}", e);
                dict.set("success", false);
                dict.set("error", e);
            }
        }
        
        dict
    }
}
