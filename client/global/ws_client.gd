extends Node

var socket: WebSocketPeer
var last_state := WebSocketPeer.STATE_CLOSED

signal connected()
signal closed()
signal packet_received(packet: Global.proto.Packet)

func _physics_process(_delta: float) -> void:
	_poll()
	_update_state()
	_read_data()

func _poll() -> void:
	if socket.get_ready_state() != socket.STATE_CLOSED:
		socket.poll()

func _update_state() -> void:
	var state := socket.get_ready_state()
	if last_state != state:
		last_state = state
		if state == socket.STATE_OPEN:
			connected.emit()
		elif state == socket.STATE_CLOSED:
			closed.emit()

func _read_data() -> void:
	var packet := _get_packet()
	if packet:
		packet_received.emit(packet)

func _get_packet() -> Global.proto.Packet:
	if socket.get_available_packet_count() < 1:
		return null

	var data := socket.get_packet()

	var packet := Global.proto.Packet.new()
	var result := packet.from_bytes(data)
	if result != OK:
		printerr("Error build packet from data: %s" % data)

	return packet

func connect_to_server(url: String, tls_options: TLSOptions = null) -> int:
	socket = WebSocketPeer.new()
	var err := socket.connect_to_url(url, tls_options)
	if err != OK:
		return err

	last_state = socket.get_ready_state()

	return OK

func send(packet: Global.proto.Packet) -> int:
	var data := packet.to_bytes()
	return socket.send(data)

func close(code: int = 1000, reason: String = "") -> void:
	socket.close(code, reason)
	last_state = socket.get_ready_state()
