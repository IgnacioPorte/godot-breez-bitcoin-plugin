extends Node
class_name Breez
## Breez Lightning SDK wrapper with signals
## Wraps the Rust BreezNode and adds convenience features

# Signals
signal connected()
signal connection_failed(error: String)
signal payment_sent(invoice: String, result: Dictionary)
signal payment_received(amount: int, description: String)
signal invoice_created(invoice: String, amount: int)
signal balance_changed(old_balance: int, new_balance: int)
signal breez_ready()

# Internal Rust node
var _breez_rust: BreezNode  # The actual Rust binding
var _last_balance: int = 0
var _is_monitoring: bool = false
var _timer: Timer

# State
var initialized := false

# Config
@export var auto_monitor_payments: bool = true
@export var check_interval: float = 2.0

func _ready():
	# Create the Rust BreezNode
	_breez_rust = BreezNode.new()
	add_child(_breez_rust)
	
	# Setup monitoring timer
	if auto_monitor_payments:
		_timer = Timer.new()
		_timer.wait_time = check_interval
		_timer.timeout.connect(_check_for_changes)
		add_child(_timer)
	
	print("Breez Node ready!")

## Initialize/Connect to Lightning Network
func init(config: Dictionary) -> bool:
	var mnemonic = config.get("mnemonic", "")
	var api_key = config.get("api_key", "")
	var network = config.get("network", "mainnet")
	var storage_dir = config.get("storage_dir", "./breez_data")
	
	return connect_to_network(mnemonic, api_key, network, storage_dir)

## Connect to network (modern API)
func connect_to_network(mnemonic: String, api_key: String, network: String = "mainnet", storage_dir: String = "./breez_data") -> bool:
	print("[Breez] Connecting to network...")
	
	var success = _breez_rust.connect_sdk(mnemonic, api_key, network, storage_dir)
	
	if success:
		initialized = true
		_last_balance = get_balance()
		emit_signal("connected")
		emit_signal("breez_ready")
		
		if auto_monitor_payments and _timer:
			_timer.start()
		
		print("[Breez] ✅ Connected successfully")
	else:
		emit_signal("connection_failed", "Failed to connect to Breez SDK")
		print("[Breez] ❌ Connection failed")
	
	return success

## Get balance in satoshis
func get_balance() -> int:
	if not initialized:
		return 0
	return _breez_rust.get_balance()

## Create Lightning invoice
func create_invoice(amount: int, description: String) -> String:
	if not initialized:
		push_error("Breez SDK not initialized")
		return ""
	
	print("[Breez] Creating invoice: %d sats" % amount)
	var invoice = _breez_rust.create_invoice(amount, description)
	
	if invoice != "":
		emit_signal("invoice_created", invoice, amount)
		print("[Breez] ✅ Invoice created")
	else:
		print("[Breez] ❌ Failed to create invoice")
	
	return invoice

## Pay a Lightning invoice
func pay_invoice(invoice: String, timeout: int = 30) -> Dictionary:
	if not initialized:
		push_error("Breez SDK not initialized")
		return {"success": false, "error": "Not initialized"}
	
	print("[Breez] Paying invoice...")
	var result = _breez_rust.pay_invoice(invoice, timeout)
	
	if result.get("success", false):
		emit_signal("payment_sent", invoice, result)
		print("[Breez] ✅ Payment sent")
	else:
		print("[Breez] ❌ Payment failed: ", result.get("error", "Unknown"))
	
	return result

## Get Bitcoin on-chain address
func get_bitcoin_address() -> String:
	if not initialized:
		return ""
	return _breez_rust.get_bitcoin_address()

## Get Spark address
func get_spark_address() -> String:
	if not initialized:
		return ""
	return _breez_rust.get_spark_address()

## Check if SDK is connected
func is_sdk_connected() -> bool:
	return initialized and _breez_rust.is_sdk_connected()

## Start monitoring for payments
func start_monitoring():
	if _timer and not _is_monitoring:
		_is_monitoring = true
		_timer.start()

## Stop monitoring
func stop_monitoring():
	if _timer and _is_monitoring:
		_is_monitoring = false
		_timer.stop()

## Disconnect from SDK
func disconnect_sdk():
	if initialized:
		if _timer:
			_timer.stop()
		_breez_rust.disconnect_breez()
		initialized = false
		print("[Breez] Disconnected")

## Utility: Format satoshis
func format_sats(amount: int) -> String:
	return "%s sats" % _format_number(amount)

## Utility: Format BTC
func format_btc(amount: int) -> String:
	return "₿ %.8f" % (amount / 100_000_000.0)

## Check for balance changes (auto-detect payments)
func _check_for_changes():
	if not initialized:
		return
	
	var current_balance = get_balance()
	
	if current_balance != _last_balance:
		# Payment received
		if current_balance > _last_balance:
			var received = current_balance - _last_balance
			emit_signal("payment_received", received, "")
			print("[Breez] 💰 Payment received: +%d sats" % received)
		
		# Balance changed
		emit_signal("balance_changed", _last_balance, current_balance)
		_last_balance = current_balance

func _format_number(num: int) -> String:
	var s = str(num)
	var result = ""
	var count = 0
	for i in range(s.length() - 1, -1, -1):
		if count == 3:
			result = "," + result
			count = 0
		result = s[i] + result
		count += 1
	return result

func _exit_tree():
	if _timer:
		_timer.stop()
	if initialized:
		disconnect_sdk()
