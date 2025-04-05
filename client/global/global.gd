extends Node

const proto := preload("res://proto.gd")
const debug_server_url := "ws://127.0.0.1:8080"
const release_server_url := "wss://agarust-server.d8s.fun"

var server_url := debug_server_url
var connection_id := ""
var show_server_position := false


func _ready() -> void:
	if not OS.is_debug_build():
		server_url = release_server_url
	WsClient.closed.connect(_on_ws_closed)


func _on_ws_closed() -> void:
	get_tree().change_scene_to_file("res://view/connecting/connecting.tscn")
