extends Node2D

@onready var username_edit: LineEdit = %UsernameEdit
@onready var password_edit: LineEdit = %PasswordEdit
@onready var login_button: Button = %LoginButton
@onready var register_button: Button = %RegisterButton
@onready var leaderboard_button: Button = %LeaderboardButton
@onready var logger: Logger = %Logger
@onready var connection_id_label: Label = %ConnectionIdLabel
@onready var message_panel: Panel = %MessagePanel
@onready var message_label: Label = %MessageLabel


func _ready() -> void:
	WsClient.packet_received.connect(_on_ws_packet_received)
	login_button.pressed.connect(_on_login_button_pressed)
	register_button.pressed.connect(_on_register_button_pressed)
	leaderboard_button.pressed.connect(_on_leaderboard_button_pressed)
	connection_id_label.text = "Connection ID: %s" % [Global.connection_id]


func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	if packet.has_login_ok():
		message_panel.hide()
		get_tree().change_scene_to_file("res://view/game/game.tscn")
	elif packet.has_login_err():
		message_panel.hide()
		logger.error(packet.get_login_err().get_reason())


func _on_login_button_pressed() -> void:
	var username := username_edit.text.strip_edges()
	var password := password_edit.text.strip_edges()
	if username.is_empty() or password.is_empty():
		return

	message_panel.show()
	message_label.text = "Loading..."

	var packet := Global.proto.Packet.new()
	var login_message := packet.new_login()
	login_message.set_username(username)
	login_message.set_password(password)
	WsClient.send(packet)


func _on_register_button_pressed() -> void:
	get_tree().change_scene_to_file("res://view/register/register.tscn")


func _on_leaderboard_button_pressed() -> void:
	get_tree().change_scene_to_file("res://view/leaderboard_view/leaderboard_view.tscn")
