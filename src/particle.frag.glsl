#version 450

in PerVertexData {
	// vec2 uv;
	float affect;
} vert_in;

perprimitiveNV in PerPrimitiveData {
	vec3 color;
} prim_in;

layout(location = 0) out vec4 out_color;

void main() {
    // out_color = vec4(vert_in.color * prim_in.color, 1.0);
    // out_color = vec4(vert_in.color, 1.0);
	// float r = length(vert_in.uv);
	// if (r > 0.45) {
	// 	discard;
	// }

	// r /= 0.45;
	// r = 1.0 - r;
	// r *= r;

    // out_color = vec4(prim_in.color * vec3(vert_in.uv + vec2(0.5), 1.0), 1.0);
	vec3 color = prim_in.color;

	// color = mix(vec3(1.0), color, r * 1.3);
	color = mix(vec3(1.0), color, vert_in.affect);


    out_color = vec4(color, 1.0);
}
