extends Node2D

@export
var game_scene: PackedScene

@onready var username_edit: LineEdit = %UsernameEdit
@onready var password_edit: LineEdit = %PasswordEdit
@onready var password_edit_2: LineEdit = %PasswordEdit2
@onready var color_picker: ColorPicker = %ColorPicker
@onready var register_button: Button = %RegisterButton
@onready var back_button: Button = %BackButton
@onready var logger: Logger = %Logger
@onready var connection_id_label: Label = %ConnectionIdLabel
@onready var message_panel: Panel = %MessagePanel
@onready var message_label: Label = %MessageLabel

func _ready() -> void:
	WsClient.packet_received.connect(_on_ws_packet_received)
	register_button.pressed.connect(_on_register_button_pressed)
	back_button.pressed.connect(_on_back_button_pressed)
	connection_id_label.text = "Connection ID: %s" % [Global.connection_id]

func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	if packet.has_register_ok():
		message_panel.hide()
		logger.success("register success")
	elif packet.has_register_err():
		message_panel.hide()
		logger.error(packet.get_register_err().get_reason())

func _on_register_button_pressed() -> void:
	var username := username_edit.text.strip_edges()
	var password := password_edit.text.strip_edges()
	var password2 := password_edit_2.text.strip_edges()
	if username.is_empty() or password.is_empty():
		return
	if password != password2:
		logger.error("passwords do not match")
		return

	message_panel.show()
	message_label.text = "Loading..."

	var packet := Global.proto.Packet.new()
	var register_message := packet.new_register()
	register_message.set_username(username)
	register_message.set_password(password)
	register_message.set_color(color_picker.color.to_rgba64())
	WsClient.send(packet)

func _on_back_button_pressed() -> void:
	get_tree().change_scene_to_file("res://view/login/login.tscn")
