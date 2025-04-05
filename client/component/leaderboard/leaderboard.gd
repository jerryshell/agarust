class_name Leaderboard
extends ScrollContainer

var _score_list: Array[int]

@onready var list_container: VBoxContainer = %ListContainer
@onready var entry_teamplate: HBoxContainer = %EntryTeamplate


func _ready() -> void:
	entry_teamplate.hide()


func _add_entry(player_nickname: String, score: int, highlight: bool) -> void:
	_score_list.append(score)
	_score_list.sort()
	var pos := len(_score_list) - _score_list.find(score) - 1

	var entry: HBoxContainer = entry_teamplate.duplicate()
	var name_label: Label = entry.get_child(0)
	var score_label: Label = entry.get_child(1)

	list_container.add_child(entry)

	list_container.move_child(entry, pos)

	name_label.text = player_nickname
	score_label.text = str(score)
	if highlight:
		name_label.add_theme_color_override("font_color", Color.YELLOW)

	entry.show()


func remove(player_nickname: String) -> void:
	for i in range(len(_score_list)):
		var entry: HBoxContainer = list_container.get_child(i)
		var name_label: Label = entry.get_child(0)
		if name_label.text == player_nickname:
			_score_list.remove_at(len(_score_list) - i - 1)
			entry.free()
			return


func set_score(player_nickname: String, score: int, highlight: bool = false) -> void:
	remove(player_nickname)
	_add_entry(player_nickname, score, highlight)


func clear() -> void:
	_score_list.clear()
	for entry in list_container.get_children():
		if entry != entry_teamplate:
			entry.free()
