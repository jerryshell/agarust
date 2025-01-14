extends Node2D

@onready var back_button: Button = %BackButton
@onready var leaderboard: Leaderboard = %Leaderboard

func _ready() -> void:
	WsClient.packet_received.connect(_on_ws_packet_received)
	back_button.pressed.connect(_on_back_button_pressed)
	_fetch_leaderboard()

func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	print_debug(packet)
	if packet.has_leaderboard_response():
		var leaderboard_response = packet.get_leaderboard_response()
		for entry: Global.proto.LeaderboardEntry in leaderboard_response.get_leaderboard_entry_list():
			var player_nickname := entry.get_player_nickname()
			var rank_and_name := "%d. %s" % [entry.get_rank(), player_nickname]
			var score: int = entry.get_score()
			leaderboard.set_score(rank_and_name, score)
	else:
		print_debug("unknown packet: ", packet)

func _fetch_leaderboard() -> void:
	var packet := Global.proto.Packet.new()
	packet.new_leaderboard_request()
	WsClient.send(packet)

func _on_back_button_pressed() -> void:
	get_tree().change_scene_to_file("res://view/login/login.tscn")
