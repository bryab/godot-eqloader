/// EQ Shader 19: Torch.  Just additive * 2
shader_type spatial;
render_mode blend_add;
render_mode unshaded;
render_mode vertex_lighting;
uniform int user_defined;
uniform sampler2D diffuse: source_color;

void fragment() {
	vec4 tex = texture(diffuse, UV);
	ALBEDO = tex.rgb * 2.0;
}