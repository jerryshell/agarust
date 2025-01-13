class_name Actor
extends Area2D

const ACTOR = preload("res://component/actor/actor.tscn")

@onready var collision_shape: CollisionShape2D = %CollisionShape
@onready var nameplate: Label = %Nameplate
@onready var camera: Camera2D = %Camera

var connection_id: String
var actor_nickname: String
var start_x: float
var start_y: float
var start_radius: float
var speed: float
var color: Color
var is_player: bool

var direction: Vector2
var radius: float:
	set(new_radius):
		radius = new_radius
		collision_shape.shape.set_radius(radius)
		_update_zoom()
		queue_redraw()

var target_zoom := 2.0
var furthest_zoom_allowed := target_zoom

var server_position: Vector2

static func instantiate(p_connection_id: String, p_actor_nickname: String, p_start_x: float, p_start_y: float, p_start_radius: float, p_speed: float, p_color: Color, p_is_player: bool) -> Actor:
	var actor := ACTOR.instantiate()
	actor.connection_id = p_connection_id
	actor.actor_nickname = p_actor_nickname
	actor.start_x = p_start_x
	actor.start_y = p_start_y
	actor.start_radius = p_start_radius
	actor.speed = p_speed
	actor.color = p_color
	actor.is_player = p_is_player
	return actor

func _ready():
	position.x = start_x
	position.y = start_y
	server_position = position
	direction = Vector2.RIGHT
	radius = start_radius
	collision_shape.shape.radius = radius
	nameplate.text = actor_nickname
	camera.enabled = is_player

func _process(_delta: float) -> void:
	if not is_equal_approx(camera.zoom.x, target_zoom):
		camera.zoom -= Vector2(1, 1) * (camera.zoom.x - target_zoom) * 0.05

func _physics_process(delta) -> void:
	position += direction * speed * delta
	server_position += direction * speed * delta
	position = position.lerp(server_position, 0.05)

	if not is_player:
		return

	var mouse_position := get_global_mouse_position()

	var distance_squared_to_mouse := position.distance_squared_to(mouse_position)
	if distance_squared_to_mouse < pow(radius, 2):
		return

	var direction_to_mouse := position.direction_to(mouse_position).normalized()

	var angle_diff = abs(direction.angle_to(direction_to_mouse))
	if angle_diff > TAU / 32:
		direction = direction_to_mouse
		_send_direction_angle()

func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, Color.BLUE_VIOLET)
	if Global.show_server_position:
		draw_circle(server_position - position, radius, Color.WHEAT)

func _input(event):
	if not is_player:
		return

	const zoom_speed := 0.1

	if event.is_action_pressed("zoom_in"):
			target_zoom = min(4, target_zoom + zoom_speed)
	elif event.is_action_pressed("zoom_out"):
			target_zoom = max(furthest_zoom_allowed, target_zoom - zoom_speed)
	camera.zoom.y = camera.zoom.x

func _send_direction_angle():
		var packet := Global.proto.Packet.new()
		var update_player_direction_angle := packet.new_update_player_direction_angle()
		update_player_direction_angle.set_direction_angle(direction.angle())
		WsClient.send(packet)

func _update_zoom() -> void:
	if is_node_ready():
		nameplate.add_theme_font_size_override("font_size", max(16, radius / 2))

	if not is_player:
		return

	var new_furthest_zoom_allowed := 2 * start_radius / radius
	if is_equal_approx(target_zoom, furthest_zoom_allowed):
		target_zoom = new_furthest_zoom_allowed
	furthest_zoom_allowed = new_furthest_zoom_allowed
