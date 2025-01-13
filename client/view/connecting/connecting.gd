extends Node2D

@export
var login_scene: PackedScene

func _ready() -> void:
	WsClient.connect_to_server(Global.server_url)
	WsClient.connected.connect(_on_ws_connected)
	WsClient.packet_received.connect(_on_ws_packet_received)

func _on_ws_connected() -> void:
	print_debug("server connected")
	get_tree().change_scene_to_packed(login_scene)

func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	print_debug(packet)
	if packet.has_hello():
		_handle_hello_msg(packet.get_hello())

func _handle_hello_msg(hello_msg: Global.proto.Hello) -> void:
	Global.connection_id = hello_msg.get_connection_id()
