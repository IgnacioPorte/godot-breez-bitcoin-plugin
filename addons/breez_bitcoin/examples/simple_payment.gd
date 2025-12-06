extends Node
## Simple Breez Lightning payment example
## This demonstrates the basic workflow: connect → create invoice → check balance

var breez: Breez

# Configuration - REPLACE THESE WITH YOUR VALUES
const MNEMONIC = "your twelve word mnemonic phrase goes here for testing"
const API_KEY = "YOUR_BREEZ_API_KEY"
const NETWORK = "regtest"  # or "mainnet" for production
const STORAGE_DIR = "./breez_data"

func _ready():
	print("=== Breez Lightning Simple Payment Example ===\n")
	setup_breez()

func setup_breez():
	# Create the Breez node
	breez = Breez.new()
	breez.auto_monitor_payments = true  # Auto-detect incoming payments
	add_child(breez)
	
	# Connect signals
	breez.connected.connect(_on_connected)
	breez.connection_failed.connect(_on_connection_failed)
	breez.payment_received.connect(_on_payment_received)
	breez.invoice_created.connect(_on_invoice_created)
	breez.balance_changed.connect(_on_balance_changed)
	
	# Connect to the network
	print("📡 Connecting to Lightning Network...")
	breez.connect_to_network(MNEMONIC, API_KEY, NETWORK, STORAGE_DIR)

func _on_connected():
	print("✅ Connected to Lightning Network!\n")
	
	# Check initial balance
	check_balance()
	
	# Create a test invoice
	create_test_invoice()
	
	# Get addresses
	get_addresses()

func _on_connection_failed(error: String):
	print("❌ Connection failed: ", error)
	print("\nTroubleshooting:")
	print("1. Check your mnemonic is correct")
	print("2. Check your API key is valid")
	print("3. Check network matches your wallet (mainnet/regtest)")

func check_balance():
	var balance = breez.get_balance()
	print("💰 Current Balance:")
	print("   ", breez.format_sats(balance))
	print("   ", breez.format_btc(balance))
	print()

func create_test_invoice():
	print("📄 Creating test invoice for 1,000 sats...")
	var invoice = breez.create_invoice(1000, "Test payment from Godot")
	
	if invoice != "":
		print("✅ Invoice created successfully")
		print("   Copy this invoice and pay it from a Lightning wallet:")
		print("   ", invoice)
		print()
		print("   Monitoring for payment...")
	else:
		print("❌ Failed to create invoice")

func get_addresses():
	print("📍 Payment Addresses:")
	
	# Bitcoin on-chain address
	var btc_address = breez.get_bitcoin_address()
	if btc_address != "":
		print("   Bitcoin: ", btc_address)
	
	# Spark address
	var spark_address = breez.get_spark_address()
	if spark_address != "":
		print("   Spark: ", spark_address)
	
	print()

func _on_invoice_created(invoice: String, amount: int):
	print("📄 Invoice Event: Created for ", amount, " sats")

func _on_payment_received(amount: int, description: String):
	print("\n🎉 PAYMENT RECEIVED!")
	print("   Amount: ", breez.format_sats(amount))
	print("   Description: ", description if description != "" else "(none)")
	print()
	
	# Show new balance
	check_balance()

func _on_balance_changed(old_balance: int, new_balance: int):
	var diff = new_balance - old_balance
	if diff > 0:
		print("💰 Balance increased by ", breez.format_sats(diff))
	elif diff < 0:
		print("💸 Balance decreased by ", breez.format_sats(abs(diff)))

func _input(event):
	if event is InputEventKey and event.pressed:
		match event.keycode:
			KEY_B:
				# Check balance
				print("\n--- Balance Check ---")
				check_balance()
			KEY_I:
				# Create new invoice
				print("\n--- Creating New Invoice ---")
				var amount = randi_range(100, 5000)
				var invoice = breez.create_invoice(amount, "Random test: %d" % amount)
				if invoice != "":
					print("Invoice: ", invoice)
			KEY_Q:
				# Quit
				print("\nExiting...")
				breez.stop_monitoring()
				get_tree().quit()

func _exit_tree():
	if breez:
		breez.stop_monitoring()
		breez.disconnect_sdk()

