// Simple example EQ shader that can handle basic opaque materials and cutout transparency (trees)

shader_type spatial;
render_mode unshaded;

uniform sampler2D diffuse: source_color;
// The 'key color' which should be transparent.  This is stored as the first color in the BMP index
uniform vec4 key_color: source_color;
uniform int shader_type_id;
const float key_tolerance = 0.001;

vec4 toLinear(vec4 sRGB)
{
    bvec4 cutoff = lessThan(sRGB, vec4(0.04045));
    vec4 higher = pow((sRGB + vec4(0.055))/vec4(1.055), vec4(2.4));
    vec4 lower = sRGB/vec4(12.92);

    return mix(higher, lower, cutoff);
}

void fragment() {
	vec4 tex = texture(diffuse, UV);
	ALPHA_SCISSOR_THRESHOLD = 1.0;
	ALBEDO = tex.rgb;
	if (shader_type_id == 19){ // TREE
		// FIXME: There is a float precision issue between the sampled key color from the BMP and the color on the GPU
		//if(tex.rgb == key_color.rgb) {
		if((tex.r >= key_color.r-key_tolerance && tex.r <= key_color.r+key_tolerance) && (tex.g >= key_color.g-key_tolerance && tex.g <= key_color.g+key_tolerance) && (tex.b >= key_color.b-key_tolerance && tex.b <= key_color.b+key_tolerance)){
			ALPHA = 0.0;
		}
	}
	//ALBEDO = mix(ALBEDO*daylight, ALBEDO, COLOR.g);
}