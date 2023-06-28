@tool
extends Node3D

var textures = {}
var materials = {}

func _ready():
	var thread = Thread.new()
	thread.start(load_random_zone)
	
func get_all_zone_s3ds():
	var eqdir = "./eq_data"
	var valid_files = []
	for filename in DirAccess.get_files_at(eqdir):
		if "_" in filename:
			continue
		if not filename.ends_with(".s3d"):
			continue
		valid_files.append(eqdir + "/" + filename)
	return valid_files

func load_random_zone():
	var all_zone_s3ds = get_all_zone_s3ds()
	var filename = all_zone_s3ds.pick_random()
	test_zone_s3d(filename)
	
func test_zone_s3d(s3d_filename):

	var loader = EQArchiveLoader.new()
	var archive = loader.load_archive(s3d_filename)
	
	
	# First load all the textures and store them in a dictionary
	for filename in archive.get_filenames():
		if filename.ends_with(".bmp"):
			textures[filename] = archive.get_texture(filename)

	# Now get the main zone WLD
	var wld: S3DWld = null
	for filename in archive.get_filenames():
		if filename.ends_with(".wld"):
			wld = archive.get_wld(filename)
			break
	
	# Load all the materials and store them in a dictionary
	for material in wld.get_materials():
		if material is S3DMaterial:
			get_or_create_material(material)
	
	# Instantiate the zone meshes
	for eqmesh in wld.get_meshes():
		var mesh_inst = build_mesh(eqmesh)
		call_deferred("add_child", mesh_inst)
			

func get_or_create_material(material_fragment: S3DMaterial, force_create = false) -> Material:
	var material_name = material_fragment.name()
	if not force_create and material_name in materials:
		return materials[material_name]
	var texture_filename = material_fragment.texture_filename()
	var texture = textures[texture_filename]

	# Note: I am just using standard material here but you'd need to provide different shaders for the various shader types: `material.shader_type_id()`
	# Also note that the textures have metadata in them to identify the 'key color' for cutout transparency (not used in this example)
	var material = StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.albedo_texture = texture

	materials[material_name] = material
	return material


func build_mesh(eqmesh: S3DMesh):
	var arrays = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = eqmesh.vertices()
	arrays[Mesh.ARRAY_NORMAL] = eqmesh.normals()
	var colors = eqmesh.vertex_colors()
	if colors:
		arrays[Mesh.ARRAY_COLOR] = colors
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
	
	var mesh_inst = MeshInstance3D.new()
	mesh_inst.mesh = mesh
	mesh_inst.name = eqmesh.name()
	mesh_inst.position = eqmesh.center()
	return mesh_inst
