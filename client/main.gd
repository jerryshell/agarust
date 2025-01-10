extends Node2D

@onready var chat_edit: LineEdit = %ChatEdit
@onready var logger: Logger = %Logger

func _ready() -> void:
	WsClient.connect_to_server(Global.server_url)
	WsClient.connected.connect(_on_ws_connected)
	WsClient.packet_received.connect(_on_ws_packet_received)
	chat_edit.text_submitted.connect(_on_chat_edit_text_submited)

func _on_ws_connected() -> void:
	logger.info("Server connected")

func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	logger.info(packet.to_string())

func _on_chat_edit_text_submited(new_text: String):
	var packet := Global.proto.Packet.new()
	var chat := packet.new_chat()
	chat.set_msg(new_text)
	WsClient.send(packet)
	chat_edit.text = ""
