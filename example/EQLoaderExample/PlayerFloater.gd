class_name PlayerFloater
extends CharacterBody3D

@onready var camera : Camera3D = get_node("Camera3D")
var mouse_delta = Vector2()

var GRAVITY = 100.0
var MOVE_MULTIPLIER = 10.0
var move_speed = 10.0 * MOVE_MULTIPLIER
var turn_speed = 50.0
var run_multiplier = 5.0
var look_sensitivity_x = 15.0 * PI/180
var look_sensitivity_y = -10.0 * PI/180

var noclip_mode = false

func _ready():
	toggle_noclip()
	Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)  

func _input(event):
	if event is InputEventMouseMotion:
		mouse_delta = event.relative

func _process(delta):
	var y_rot = 0
	if Input.is_action_pressed("turn_left"):
		y_rot = -.1
	elif Input.is_action_pressed("turn_right"):
		y_rot = .1
		
	y_rot += mouse_delta.x * delta * look_sensitivity_x
	
	
	# FIXME: I think looking up and down should be done on the CAMERA, not the player, except
	# when the player is floating.
	var rotation_y = y_rot * delta * turn_speed
	if noclip_mode:
		rotation.y -= rotation_y
		camera.rotation.y = 0
	else:
		camera.rotation.y -= rotation_y
		rotation.y = 0

	var x_rot = mouse_delta.y * look_sensitivity_y * delta

	rotation.x -= x_rot
	
	mouse_delta = Vector2()
	
# Called every frame. 'delta' is the elapsed time since the previous frame.
func _physics_process(delta):
	
	velocity.x = 0
	velocity.y = 0
	
	var multiplier = 1.0
	if Input.is_action_pressed("run"):
		multiplier = run_multiplier
		
	var movement = Vector2()
	
	if Input.is_action_pressed("ui_left"):
		movement.x = -1
	elif Input.is_action_pressed("ui_right"):
		movement.x = 1
	if Input.is_action_pressed("ui_up"):
		movement.y = -1
	elif Input.is_action_pressed("ui_down"):
		movement.y = 1
	
	#get the forward and right directions
	var forward = global_transform.basis.z
	var right = global_transform.basis.x
	var relativeDir = (forward * movement.y + right * movement.x)
	
	velocity.x = relativeDir.x * move_speed * multiplier
	velocity.y = relativeDir.y * move_speed * multiplier
	velocity.z = relativeDir.z * move_speed * multiplier
	
	if Input.is_action_pressed("ui_accept"):
		velocity.y = move_speed
	elif Input.is_action_pressed("crouch"):
		velocity.y = -move_speed
	
	
	if not noclip_mode:
		velocity.y -= GRAVITY * delta
	
	move_and_slide()

func toggle_noclip():
	noclip_mode = !noclip_mode
	var collider = get_node("CollisionShape3d")
	if collider:
		collider.disabled = noclip_mode
