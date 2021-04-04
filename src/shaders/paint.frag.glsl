#version 450

layout(binding = 0) uniform sampler2D u_paint_sampler;


in PerVertexData {
	vec2 uv;
} vert_in;

layout(location = 0) out vec4 out_color;



vec2 sample_paint(vec2 uv) {
	const ivec2 offsets[] = {
		ivec2(-3, 0),
		ivec2( 3, 0),
		ivec2( 0,-3),
		ivec2( 0, 3),
	};

	vec4 samples = textureGatherOffsets(u_paint_sampler, uv, offsets, 0);
	return vec2(
		samples.y - samples.x,
		samples.w - samples.z
	);
}

void main() {
	float value = texture(u_paint_sampler, vert_in.uv).r;
    // out_color = vec4(value, vert_in.uv, 1.0);
    out_color = vec4(fract(value));
    // out_color = vec4(vec3(sample_paint(vert_in.uv) * 0.5 + 0.5, 0.0), 1.0);
}
