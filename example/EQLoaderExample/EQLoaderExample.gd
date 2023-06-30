extends Node3D

var standard_shader: Shader = preload("res://shaders/eq_standard.gdshader")
var textures = {}
var materials = {}
var actordefs = {}

func _ready():
	var thread = Thread.new()
	thread.start(load_random_zone)
	thread.wait_to_finish()

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
			textures[filename] = archive.get_texture(filename)

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
		
func create_material(material_fragment: S3DMaterial, force_create = false) -> Material:
	var material_name = material_fragment.name()
	if not force_create and material_name in materials:
		return materials[material_name]
	var texture_filename = material_fragment.texture_filename()
	var texture = textures[texture_filename]

	# Note: I am just using standard material here but you'd need to provide different shaders for the various shader types: `material.shader_type_id()`
	# Also note that the textures have metadata in them to identify the 'key color' for cutout transparency (not used in this example)
	var material = ShaderMaterial.new()
	material.set_name(material_name)
	material.shader = standard_shader
	material.set_shader_parameter("diffuse", texture)
	material.set_shader_parameter("shader_type_id", material_fragment.shader_type_id())
	# When the texture was first loaded, a metadata item was stored on it which, for cut-out transparent textures, is the color to be cut out.
	# There is a problem at the moment where the color on the GPU in the shader is slightly different from this color,
	# So in the shader I am allowing some wiggle room.
	material.set_shader_parameter("key_color", texture.get_meta("key_color"))
	materials[material_name] = material
	return material

func build_mesh(eqmesh: S3DMesh,) -> ArrayMesh:
	var arrays = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = eqmesh.vertices()
	arrays[Mesh.ARRAY_NORMAL] = eqmesh.normals()
	#if vertex_colors == null:
	var vertex_colors = eqmesh.vertex_colors()
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
		var material = materials[material_name]

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
	var mesh = build_mesh(eqmesh)
	
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
	
