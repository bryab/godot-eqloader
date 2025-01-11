@tool
extends Node3D

var builder: EQBuilder
var threads = []

func _ready():
	builder = EQBuilder.new()
	add_child(builder)
	
	var args = OS.get_cmdline_user_args()
	
	
	print("Loading...")
	for zone_name in get_all_zone_names():
		print("Found zone: %s" % [zone_name])
	
	if len(args):
		var cmd = args[0]
		if cmd == "--zone":
			var zone_name = args[1]
			run_in_thread(builder.load_zone.bind(zone_name))
			run_in_thread(load_chr.bind("%s_chr" % [zone_name]))
			return
		elif cmd == '--chr':
			var s3d_name = args[1]
			run_in_thread(load_chr.bind(s3d_name))
			return
	
	print("Loading random zone and character file.")
	run_in_thread(load_random_zone)
	run_in_thread(load_random_chr)

func run_in_thread(callable: Callable):
	var thread = Thread.new()
	thread.start(callable)
	threads.append(thread)
	
func _exit_tree():
	for thread in threads:
		thread.wait_to_finish()

# Just a quick function to get the names of all the zones in the data directory
func get_all_zone_names():
	var eqdir = builder.get_eq_data_dir()
	print("Attempting to load from dir: %s" % [eqdir])
	var names = []
	for filename in DirAccess.get_files_at(eqdir):
		if "_" in filename:
			continue
		if not filename.ends_with(".s3d"):
			continue
		var zone_name = filename.get_basename()
		if zone_name == "gequip":
			continue
		names.append(zone_name)
	return names

func get_all_chr_names():
	var eqdir = builder.get_eq_data_dir()
	var names = []
	for filename in DirAccess.get_files_at(eqdir):
		if filename.ends_with("_chr.s3d"):
			names.append(filename.get_basename())
	return names

func get_random_zone_name():
	return get_all_zone_names().pick_random()

func load_random_zone():
	builder.load_zone(get_random_zone_name())
	#call_deferred("own_children")

func load_chr(chr_name: String):
	var actor_nodes = builder.load_chr(chr_name)
	call_deferred("play_random_animations", actor_nodes)

func play_random_animations(actor_nodes):
	for actor_node in actor_nodes:
		var animation_player: AnimationPlayer = actor_node.find_child("*_ANIM", true, false)
		play_random_animation(animation_player)
		
func load_random_chr():
	load_chr(get_all_chr_names().pick_random())

func play_random_animation(animation_player: AnimationPlayer):
	if not len(animation_player.get_animation_list()):
		return
	var random_anim_name = animation_player.get_animation_list()[randi_range(0, len(animation_player.get_animation_list())) - 1]
	animation_player.play(random_anim_name)
	
