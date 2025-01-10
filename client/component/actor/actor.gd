class_name Actor
extends Area2D

const ACTOR = preload("res://component/actor/actor.tscn")

@onready var collision_shape: CollisionShape2D = %CollisionShape
@onready var nameplate: Label = %Nameplate
@onready var camera: Camera2D = %Camera

var connection_id: String
var actor_name: String
var start_x: float
var start_y: float
var start_rad: float
var speed: float
var color: Color
var is_player: bool

var velocity: Vector2
var radius: float:
	set(new_radius):
		radius = new_radius
		collision_shape.shape.set_radius(radius)
		_update_zoom()
		queue_redraw()

var target_zoom := 2.0
var furthest_zoom_allowed := target_zoom

var server_position: Vector2

static func instantiate(connection_id: String, actor_name: String, x: float, y: float, radius: float, speed: float, color: Color, is_player: bool) -> Actor:
	var actor := ACTOR.instantiate()
	actor.connection_id = connection_id
	actor.actor_name = actor_name
	actor.start_x = x
	actor.start_y = y
	actor.start_rad = radius
	actor.speed = speed
	actor.color = color
	actor.is_player = is_player
	return actor

func _ready():
	position.x = start_x
	position.y = start_y
	server_position = position
	velocity = Vector2.RIGHT * speed
	radius = start_rad
	collision_shape.shape.radius = radius
	nameplate.text = actor_name
	camera.enabled = is_player

func _process(_delta: float) -> void:
	if not is_equal_approx(camera.zoom.x, target_zoom):
		camera.zoom -= Vector2(1, 1) * (camera.zoom.x - target_zoom) * 0.05

func _physics_process(delta) -> void:
	position += velocity * delta
	server_position += velocity * delta
	position = position.lerp(server_position, 0.05)

	if not is_player:
		return

	# Player-specific stuff below here
	var mouse_pos := get_global_mouse_position()
	var input_vec = position.direction_to(mouse_pos).normalized()

	if abs(velocity.angle_to(input_vec)) > TAU / 15: # 24 degrees
		velocity = input_vec * speed

		var packet := Global.proto.Packet.new()
		var update_player_direction_angle := packet.new_update_player_direction_angle()
		update_player_direction_angle.set_direction_angle(velocity.angle())
		WsClient.send(packet)

func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, Color.BLUE_VIOLET)

func _input(event):
	if is_player:
		if event.is_action_pressed("zoom_in"):
				target_zoom = min(4, target_zoom + 0.1)
		elif event.is_action_pressed("zoom_out"):
				target_zoom = max(furthest_zoom_allowed, target_zoom - 0.1)
		camera.zoom.y = camera.zoom.x

func _update_zoom() -> void:
	if is_node_ready():
		nameplate.add_theme_font_size_override("font_size", max(16, radius / 2))

	if not is_player:
		return

	var new_furthest_zoom_allowed := 2 * start_rad / radius
	if is_equal_approx(target_zoom, furthest_zoom_allowed):
		target_zoom = new_furthest_zoom_allowed
	furthest_zoom_allowed = new_furthest_zoom_allowed
