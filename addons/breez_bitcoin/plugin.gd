@tool
extends EditorPlugin

func _enter_tree():
	add_custom_type("Breez", "Node", preload("breez_node.gd"), preload("res://icon.png"))

func _exit_tree():
	remove_custom_type("Breez")
