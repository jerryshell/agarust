extends Node2D

func _ready() -> void:
	var packet = Global.proto.Packet.new()
	packet.set_connection_id("123")
	var hello = packet.new_hello()
	hello.set_connection_id("123")
	print(packet.to_bytes())
