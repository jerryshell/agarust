class_name Spore
extends Area2D

const SPORE = preload("res://component/spore/spore.tscn")

var spore_id: String
var x: float
var y: float
var radius: float
var color: Color
var underneath_player: bool

@onready var collision_shape: CollisionShape2D = %CollisionShape

static func instantiate(p_spore_id: String, p_x: float, p_y: float, p_radius: float, p_underneath_player: bool) -> Spore:
	var spore := SPORE.instantiate()
	spore.spore_id = p_spore_id
	spore.x = p_x
	spore.y = p_y
	spore.radius = p_radius
	spore.underneath_player = p_underneath_player
	return spore

func _ready() -> void:
	if underneath_player:
		area_exited.connect(_on_area_exited)
	position.x = x
	position.y = y
	collision_shape.shape.radius = radius
	color = Color.from_hsv(randf(), 1, 1, 1)

func _draw() -> void:
	draw_circle(Vector2.ZERO, radius, color)

func _on_area_exited(area: Area2D) -> void:
	if area is Actor:
		underneath_player = false
