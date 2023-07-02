extends Node3D

var default_material = ShaderMaterial.new()
var shader_standard: Shader = preload("res://shaders/eq_standard.gdshader")
var shader_add: Shader = preload("res://shaders/eq_additive.gdshader")
var textures = {}
var materials = {}
var actordefs = {}

var library_lock = Mutex.new()

func get_shader(shader_type_id: int) -> Shader:
	if shader_type_id in [0x0B, 0x17]:
		return shader_add
	return shader_standard
	
func _ready():
	var thread = Thread.new()
	thread.start(load_random_zone)
	var thread2 = Thread.new()
	thread2.start(load_random_chr)
	thread.wait_to_finish()
	thread2.wait_to_finish()

func get_eq_data_dir():
	var eq_dir_locations = [OS.get_environment("EQDATA"), "res://eq_data"]
	for eqdir in eq_dir_locations:
		if DirAccess.open(eqdir) != null:
			return eqdir

# Just a quick function to get the names of all the zones in the data directory
func get_all_zone_names():
	var eqdir = get_eq_data_dir()
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

func load_random_zone():
	var all_zones = get_all_zone_names()
	var zone_name = all_zones.pick_random()
	load_zone(zone_name)
	
func load_archive_textures(archive):
	for filename in archive.get_filenames():
		if filename.ends_with(".bmp"):
			library_lock.lock()
			textures[filename] = archive.get_texture(filename)
			library_lock.unlock()

func load_wld_materials(wld):
	for material in wld.get_materials():
		if material is S3DMaterial:
			create_material(material)

func load_zone(zone_name):

	var eqdir = get_eq_data_dir()
	var loader = EQArchiveLoader.new()
	var archive: EQArchive = loader.load_archive("{0}/{1}.s3d".format([eqdir, zone_name]))
	
	# First load all the textures and store them in a dictionary
	load_archive_textures(archive)
	
	# Now get the main zone WLD
	var wld: S3DWld = archive.get_main_wld()
	
	# Load all the materials and store them in a dictionary
	load_wld_materials(wld)
	
	# Instantiate the zone meshes
	for eqmesh in wld.get_meshes():
		var mesh_inst: MeshInstance3D = build_mesh_inst(eqmesh)
		call_deferred("add_child", mesh_inst)

	# Now get the actordef S3D.
	# This has the definitions of all the placeable objects in the scene for the zone
	var actordef_archive: EQArchive = loader.load_archive("{0}/{1}_obj.s3d".format([eqdir, zone_name]))
	
	# First load all the textures and store them in a dictionary
	load_archive_textures(actordef_archive)
	
	# Get the WLD containing the actordefs
	var actordef_wld = actordef_archive.get_main_wld()
	
	# Load all the materials and store them in a dictionary
	load_wld_materials(actordef_wld)
	
	# Store references to the actordefs in a dictionary.
	# Note that this will retain hold of the underlying WLD data.
	# In a better system, just store the data you need from the actor (the mesh arrays)
	for actordef in actordef_wld.get_actordefs():
		actordefs[actordef.name()] = actordef
		
	# Now get the actor_instances WLD from the main S3D
	# This tells us where to place the actors.
	# Note that in the actual game, there would be additional actors which are defined on the server.
	var actorinst_wld = archive.get_actorinst_wld()
	
	# Now instantiate the actors based on the actorinstances WLD from the zone.
	for actorinst in actorinst_wld.get_actorinstances():
		var actorinst_node = build_actorinst(actorinst)
		call_deferred("add_child", actorinst_node)


func get_texture(texture_filename: String) -> Texture:
	return textures[texture_filename]
	
func get_texture_for_material(material_fragment: S3DMaterial) -> Texture:
	var texture_filenames = material_fragment.texture_filenames()
	var num_textures = len(texture_filenames)
	if num_textures > 1:
		var anim = AnimatedTexture.new()
		anim.frames = len(texture_filenames)
		for i in range(num_textures):
			anim.set_frame_texture(i, get_texture(texture_filenames[i]))
			anim.set_frame_duration(i, material_fragment.delay()) # FIXME: Where is this stored? Anywhere?
		return anim
	return get_texture(texture_filenames[0])

func create_material(material_fragment: S3DMaterial) -> Material:
	var material_name = material_fragment.name()
	
	# For invisible materials, I am setting a null value.
	# This is used later when building the mesh to skip polygons with this material.
	if not material_fragment.visible():
		materials[material_name] = null
		return
	
	var texture = get_texture_for_material(material_fragment)

	var shader_type_id = material_fragment.shader_type_id()
	# Note: I am just using standard material here but you'd need to provide different shaders for the various shader types: `material.shader_type_id()`
	# Also note that the textures have metadata in them to identify the 'key color' for cutout transparency (not used in this example)
	var material = ShaderMaterial.new()
	material.set_name(material_name)
	material.shader = get_shader(shader_type_id)
	material.set_shader_parameter("diffuse", texture)
	material.set_shader_parameter("shader_type_id", shader_type_id)
	# When the texture was first loaded, a metadata item was stored on it which, for cut-out transparent textures, is the color to be cut out.
	# There is a problem at the moment where the color on the GPU in the shader is slightly different from this color,
	# So in the shader I am allowing some wiggle room.
	if texture.has_meta("key_color"):
		material.set_shader_parameter("key_color", texture.get_meta("key_color"))
	library_lock.lock()
	materials[material_name] = material
	library_lock.unlock()
	return material

func build_mesh(eqmesh: S3DMesh, vertex_colors: PackedColorArray = []) -> ArrayMesh:
	var arrays = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = eqmesh.vertices()
	arrays[Mesh.ARRAY_NORMAL] = eqmesh.normals()
	# ActorInstances have their own vertex_colors array.
	if vertex_colors:
		arrays[Mesh.ARRAY_COLOR] = vertex_colors
	else:
		# For zone meshes, use the vertex_colors that are part of the mesh.
		vertex_colors = eqmesh.vertex_colors()
		if vertex_colors:
			arrays[Mesh.ARRAY_COLOR] = vertex_colors
	var uvs = eqmesh.uvs()
	if uvs:
		arrays[Mesh.ARRAY_TEX_UV] = uvs
	var bone_indices = eqmesh.bone_indices()
	if bone_indices:
		arrays[Mesh.ARRAY_BONES] = bone_indices
		arrays[Mesh.ARRAY_WEIGHTS] = eqmesh.bone_weights()
	
	var mesh = ArrayMesh.new()
	
	var surf_idx = 0

	for face_material_group in eqmesh.face_material_groups():
		var material_name = face_material_group[0]
		var indices = face_material_group[1]
		if len(indices) < 1:
			continue
		var material: Material = null
		if not material_name in materials:
			push_error("Missing material: %s" % [material_name])
			material = default_material
		else:
			material = materials[material_name]
		if material == null:
			# If material is null, it means it is invisible.
			# Skip the polygons entirely.
			continue

		arrays[Mesh.ARRAY_INDEX] = indices
		
		mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
		mesh.surface_set_material(surf_idx, material)
		
		surf_idx += 1
	
	return mesh
	

func build_mesh_inst(eqmesh: S3DMesh) -> MeshInstance3D:
	var mesh = build_mesh(eqmesh)
	var mesh_inst = MeshInstance3D.new()
	mesh_inst.mesh = mesh
	mesh_inst.name = eqmesh.name()
	mesh_inst.position = eqmesh.center()
	return mesh_inst

func build_actorinst(actorinst: S3DActorInstance) -> Node3D:
	var actordef: S3DActorDef = actordefs[actorinst.actordef_name()]
	# For this simple example just assume there's only one mesh.
	# Characters will have more than one (head and body) but those are handled differently than simple actors.
	var eqmesh = actordef.meshes()[0]
	# The mesh must be built for each instance, because each instance has different vertex colors.
	var mesh = build_mesh(eqmesh, actorinst.vertex_colors())
	
	# First make the mesh and position it
	var mesh_inst = MeshInstance3D.new()
	mesh_inst.mesh = mesh
	mesh_inst.name = eqmesh.name()
	mesh_inst.position = eqmesh.center()

	# Now make an empty node for the actor, so that it can be positioned and scaled with the instance settings
	var actorinst_node = Node3D.new()
	actorinst_node.add_child(mesh_inst)
	actorinst_node.position = actorinst.position()
	actorinst_node.quaternion = actorinst.quaternion()
	return actorinst_node


func load_random_chr():
	var s3d_name = "global_chr"
	var eqdir = get_eq_data_dir()
	var loader = EQArchiveLoader.new()
	var archive: EQArchive = loader.load_archive("{0}/{1}.s3d".format([eqdir, s3d_name]))
	
	# First load all the textures and store them in a dictionary
	load_archive_textures(archive)
	
	# Now get the chr WLD
	var wld: S3DWld = archive.get_main_wld()
	
	# Load all the materials and store them in a dictionary
	load_wld_materials(wld)
	
	var i = 0
	for eqskel in wld.get_skeletons():
		#print(eqskel)
		var skeleton = build_skeleton(eqskel)
		skeleton.position = Vector3(i*10, 0, 0)
		i += 1
		# Note: the skeleton must be added to the scene tree before assigning meshes to it.
		# I feel this is a bug, but perhaps I am misunderstanding something.
		call_deferred("add_child", skeleton)
		for eqmesh in eqskel.meshes():
			call_deferred("add_mesh_to_skeleton", eqmesh, skeleton)
		# Note: The character will look broken until some kind of animation is applied to the skeleton.
	
func add_mesh_to_skeleton(eqmesh: S3DMesh, skeleton: Skeleton3D) -> Skeleton3D:
	var mesh_inst = build_mesh_inst(eqmesh)
	skeleton.add_child(mesh_inst)
	mesh_inst.skeleton = skeleton.get_path()
	return skeleton
	

func build_skeleton(eqskel: S3DSkeleton) -> Skeleton3D:
	var skeleton = Skeleton3D.new()
	
	
	# First create all the bones
	for bone in eqskel.bones():
		var bone_name = bone.name()
		skeleton.add_bone(bone_name)
		
		var mesh_attachment = bone.attachment()
		if mesh_attachment:
			var bone_attachment = BoneAttachment3D.new()
			bone_attachment.name = "BONE_%s" % [bone_name]
			bone_attachment.bone_name = bone_name
			var mesh_inst = build_mesh_inst(mesh_attachment)
			bone_attachment.add_child(mesh_inst)
			skeleton.add_child(bone_attachment)
		
	
	# Then setup parenting - because the parents must exist first.
	var bone_index = 0
	for bone in eqskel.bones():
		skeleton.set_bone_parent(bone_index, bone.parent_index())
	
	return skeleton
		
	
