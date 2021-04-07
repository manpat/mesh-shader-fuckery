#version 450

layout(binding = 0) uniform sampler2D u_heightmap_sampler;

in PerVertexData {
	vec3 debug_col;
	vec2 uv;
} vert_in;


layout(location = 0) out vec4 out_color;



void main() {
	float height = texture2D(u_heightmap_sampler, vert_in.uv).r;
    out_color = vec4(vert_in.debug_col.xy, fract(height), 1.0);
}
