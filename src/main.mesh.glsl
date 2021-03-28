#version 450
#extension GL_NV_mesh_shader : require


struct Vertex {
	vec3 position;
	vec3 color;
};

struct Meshlet {
	uint vertex_count;
	uint primitive_count;
	uint vertex_begin;
	uint primitive_begin;
};

layout(std140, binding = 2) uniform UniformData {
	layout(row_major) mat4 u_projection_view;
};

layout(std430, binding = 3) buffer VertexData {
	Vertex vertices[];
};

layout(std430, binding = 4) buffer MeshletData {
	uint meshlet_data[];
};


out PerVertexData {
	vec3 color;
} vert_out[];

perprimitiveNV out PerPrimitiveData {
	vec3 color;
} prim_out[];



Meshlet fetch_meshlet(uint index) {
	index *= 4; // sizeof Meshlet
	index += 2; // skip Header

	return Meshlet(
		meshlet_data[index + 0],
		meshlet_data[index + 1],
		meshlet_data[index + 2],
		meshlet_data[index + 3]
	);
}

uint fetch_vertex_index(uint index) {
	uint vertex_indices_offset = meshlet_data[0];
	uint packed_value = meshlet_data[vertex_indices_offset + index / 2];
	int which = int(index) % 2;
	return bitfieldExtract(packed_value, which * 16, 16);
}

uvec3 fetch_triangle_indices(uint index) {
	uint primitive_indices_offset = meshlet_data[1];
	uint packed_a = meshlet_data[primitive_indices_offset + index * 3 / 4];
	uint packed_b = meshlet_data[primitive_indices_offset + index * 3 / 4 + 1];

	uvec4 unpacked_a = uvec4(
		bitfieldExtract(packed_a, 0 * 8, 8),
		bitfieldExtract(packed_a, 1 * 8, 8),
		bitfieldExtract(packed_a, 2 * 8, 8),
		bitfieldExtract(packed_a, 3 * 8, 8)
	);

	uvec2 unpacked_b = uvec2(
		bitfieldExtract(packed_b, 0 * 8, 8),
		bitfieldExtract(packed_b, 1 * 8, 8)
	);

	uint which = index * 3 % 4;

	uvec3 options[] = {
		unpacked_a.xyz,
		unpacked_a.yzw,
		uvec3(unpacked_a.zw, unpacked_b.x),
		uvec3(unpacked_a.w, unpacked_b.xy),
	};

	return options[which];
}


const vec3 colors[] = {
	vec3(1.0, 0.5, 1.0),
	vec3(0.5, 1.0, 1.0),
	vec3(1.0, 1.0, 0.5),
	vec3(1.0, 0.5, 0.5),
	vec3(0.5, 0.5, 1.0),
	vec3(0.5, 1.0, 0.5),
	vec3(1.0, 1.0, 1.0),
};

layout(triangles) out;
layout(local_size_x=1) in;
layout(max_vertices=64, max_primitives=126) out;

void main() {
	uint meshlet_id = gl_WorkGroupID.x;
	uint local_id = gl_LocalInvocationID.x;

	Meshlet meshlet = fetch_meshlet(meshlet_id);

	for (int v = 0; v < meshlet.vertex_count; v++){
		uint vertex_index = fetch_vertex_index(meshlet.vertex_begin + v).x;
		Vertex vertex = vertices[vertex_index];

		gl_MeshVerticesNV[v].gl_Position = u_projection_view * vec4(vertex.position, 1.0);
		vert_out[v].color = vertex.color;
	}

	for (int p = 0; p < meshlet.primitive_count; p++){
		uvec3 triangle = fetch_triangle_indices(meshlet.primitive_begin + p);
		gl_PrimitiveIndicesNV[p * 3 + 0] = triangle.x;
		gl_PrimitiveIndicesNV[p * 3 + 1] = triangle.y;
		gl_PrimitiveIndicesNV[p * 3 + 2] = triangle.z;
		prim_out[p].color = colors[meshlet_id % colors.length()];
	}

	if (local_id == 0) {
		gl_PrimitiveCountNV = meshlet.primitive_count;
	}
}