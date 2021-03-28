#version 450

in PerVertexData {
	vec3 color;
} vert_in;

perprimitiveNV in PerPrimitiveData {
	vec3 color;
} prim_in;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = vec4(vert_in.color * prim_in.color, 1.0);
    // out_color = vec4(vert_in.color, 1.0);
}
