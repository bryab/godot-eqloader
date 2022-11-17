extends Node

# Called when the node enters the scene tree for the first time.
func _ready():
	var loader = EQArchiveLoader.new()
	add_child(loader)
