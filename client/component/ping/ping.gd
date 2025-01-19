extends Label

@onready var timer: Timer = %Timer

func _ready() -> void:
	WsClient.packet_received.connect(_on_ws_packet_received)
	timer.timeout.connect(_on_timer_timeout)

func _on_ws_packet_received(packet: Global.proto.Packet) -> void:
	if packet.has_ping():
		var ping := packet.get_ping()
		var client_timestamp := ping.get_client_timestamp()
		var now := int(Time.get_unix_time_from_system() * 1000)
		var diff := now - client_timestamp
		text = "Ping: %s ms" % diff

func _on_timer_timeout() -> void:
	var packet := Global.proto.Packet.new()
	var ping := packet.new_ping()
	var now := int(Time.get_unix_time_from_system() * 1000)
	ping.set_client_timestamp(now)
	WsClient.send(packet)
