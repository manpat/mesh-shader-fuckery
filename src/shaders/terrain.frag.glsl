#version 450

// layout(binding = 0) uniform sampler2D u_paint_sampler;


in PerVertexData {
	vec2 uv;
} vert_in;

layout(location = 0) out vec4 out_color;



void main() {
    out_color = vec4(vert_in.uv, 0.5, 1.0);
}
