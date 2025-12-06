# ⚡ Breez Lightning for Godot

A Godot plugin for integrating Lightning Network payments using the Breez Spark SDK.

## About

**Breez SDK** is a self-custodial Lightning Network solution that enables developers to add Bitcoin payments to their applications. This plugin brings Breez's Lightning capabilities directly into Godot, allowing you to accept payments in your games without relying on third-party custodians.

With this plugin, you can create Lightning invoices, monitor incoming payments, check balances, and more - all with a few lines of GDScript. The SDK handles the complexity of Lightning Network channels, routing, and on-chain operations.

## Features

- ⚡ Create and pay Lightning invoices (BOLT11)
- 💰 Real-time balance checking
- 📡 Payment monitoring with signals
- 🔗 Bitcoin on-chain addresses
- 🌟 Spark address support

## Quick Start

### 1. Installation

1. Download or copy the `addons/breez_bitcoin/` folder to your Godot project
2. In Godot, go to **Project → Project Settings → Plugins**
3. Find **"Breez Lightning"** and check **Enable**
4. The custom **Breez** node will now be available in your scene tree

### 2. Usage

**📚 See the complete example:** [`addons/breez_bitcoin/examples/simple_payment.gd`](addons/breez_bitcoin/examples/simple_payment.gd)


## API Reference

### Signals

- `connected()` - Connected to Lightning Network
- `payment_received(amount: int, description: String)` - Payment received
- `payment_sent(invoice: String, result: Dictionary)` - Payment sent
- `invoice_created(invoice: String, amount: int)` - Invoice created
- `balance_changed(old: int, new: int)` - Balance changed

### Methods

#### `connect_to_network(mnemonic, api_key, network, storage_dir) -> bool`
Connect to Lightning Network.

**Parameters:**
- `mnemonic` - 12 or 24 word BIP39 phrase
- `api_key` - Breez API key ([get one here](https://breez.technology))
- `network` - "mainnet" or "regtest"
- `storage_dir` - Storage directory path

#### `get_balance() -> int`
Get balance in satoshis.

#### `create_invoice(amount: int, description: String) -> String`
Create Lightning invoice.

#### `pay_invoice(invoice: String, timeout: int) -> Dictionary`
Pay Lightning invoice.

#### `get_bitcoin_address() -> String`
Get Bitcoin on-chain address.

#### `get_spark_address() -> String`
Get Spark address.

#### `is_sdk_connected() -> bool`
Check if connected.

#### `format_sats(amount: int) -> String`
Format satoshis with commas (e.g., "1,000 sats").


## Requirements

- Godot 4.1 or higher
- Breez API key (get one [here](https://breez.technology))
- Internet connection (for Lightning Network)