extends Node2D

@onready var world: Node2D = %World
@onready var chat_edit: LineEdit = %ChatEdit
@onready var logger: Logger = %Logger

# connection_id -> player
var player_map: Dictionary = {}

func _ready() -> void:
	WsClient.connect_to_server(Global.server_url)
	WsClient.connected.connect(_on_ws_connected)
	WsClient.packet_received.connect(_on_ws_packet_received)
	chat_edit.text_submitted.connect(_on_chat_edit_text_submited)

func _on_ws_connected() -> void:
	logger.info("Server connected")

func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	print(packet)
	var connection_id = packet.get_connection_id()
	if packet.has_hello():
		_handle_hello_msg(connection_id, packet.get_hello())
	elif packet.has_chat():
		_handle_chat_msg(connection_id, packet.get_chat())
	elif packet.has_update_player():
		_handle_update_player_msg(connection_id, packet.get_update_player())

func _on_chat_edit_text_submited(new_text: String):
	var packet := Global.proto.Packet.new()
	var chat := packet.new_chat()
	chat.set_msg(new_text)
	WsClient.send(packet)
	chat_edit.text = ""

func _handle_hello_msg(connection_id: String, chat_msg: Global.proto.Hello) -> void:
	Global.connection_id = connection_id

func _handle_chat_msg(connection_id: String, chat_msg: Global.proto.Chat) -> void:
	logger.chat(connection_id, chat_msg.get_msg())

func _handle_update_player_msg(connection_id: String, update_player_msg: Global.proto.UpdatePlayer) -> void:
	var actor_name := update_player_msg.get_name()
	var x := update_player_msg.get_x()
	var y := update_player_msg.get_y()
	var radius := update_player_msg.get_radius()
	var speed := update_player_msg.get_speed()
	var color_hex := update_player_msg.get_color()

	var color := Color.hex(color_hex)
	var is_player := connection_id == Global.connection_id

	if connection_id not in player_map:
		_add_actor(connection_id, actor_name, x, y, radius, speed, color, is_player)
	else:
		var direction := update_player_msg.get_direction()
		_update_actor(connection_id, x, y, direction, speed, radius, is_player)

func _add_actor(connection_id: String, actor_name: String, x: float, y: float, radius: float, speed: float, color: Color, is_player: bool) -> void:
	var actor := Actor.instantiate(connection_id, actor_name, x, y, radius, speed, color, is_player)
	actor.z_index = 1
	world.add_child(actor)
	_set_actor_mass(actor, _rad_to_mass(radius))
	player_map[connection_id] = actor

	if is_player:
		actor.area_entered.connect(_on_player_area_entered)

func _rad_to_mass(radius: float) -> float:
	return radius * radius * PI

func _set_actor_mass(actor: Actor, new_mass: float) -> void:
	actor.radius = sqrt(new_mass / PI)
	#hiscores.set_hiscore(actor.actor_name, roundi(new_mass))

func _on_player_area_entered(area: Area2D) -> void:
	print("_on_player_area_entered ", area)
	#if area is Spore:
		#_consume_spore(area as Spore)
	#elif area is Actor:
		#_collide_actor(area as Actor)

func _update_actor(connection_id: String, x: float, y: float, direction: float, speed: float, radius: float, is_player: bool) -> void:
	var actor = player_map[connection_id]
	_set_actor_mass(actor, _rad_to_mass(radius))
	actor.radius = radius

	var server_position := Vector2(x, y)
	if actor.position.distance_squared_to(server_position) > 50:
		actor.server_position = server_position

	if not is_player:
		actor.velocity = Vector2.from_angle(direction) * speed
