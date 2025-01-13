extends Node

const proto := preload("res://proto.gd")
const server_url := "ws://127.0.0.1:8080"

var connection_id := ""

func _ready() -> void:
	WsClient.closed.connect(_on_ws_closed)

func _on_ws_closed() -> void:
	get_tree().change_scene_to_file("res://view/connecting/connecting.tscn")
