#version 450
#extension GL_NV_mesh_shader : require

#import global

layout(triangles) out;
layout(local_size_x=32) in;
layout(max_vertices=64, max_primitives=126) out;


layout(binding = 0) uniform sampler2D u_heightmap_sampler;

out PerVertexData {
	vec3 debug_col;
	vec2 uv;
} vert_out[];


const uvec2 index_offsets[] = {
	uvec2(0, 0),
	uvec2(1, 0),
	uvec2(0, 1),

	uvec2(1, 0),
	uvec2(1, 1),
	uvec2(0, 1),
};


vec2 calculate_vertex(uint total_vert_span, uint index) {
	ivec2 index_2d = ivec2(index % total_vert_span, index / total_vert_span); 
	return vec2(index_2d);
}

vec2 calculate_patch_offset(uint total_patch_span, uint index) {
	ivec2 index_2d = ivec2(index % total_patch_span, index / total_patch_span); 
	return vec2(index_2d);
}

uvec3 calculate_indices(uint total_quad_span, uint primitive_idx) {
	const uint quad_idx = primitive_idx / 2;
	const uint which = primitive_idx % 2;

	const uint vert_span = total_quad_span + 1;

	const uint quad_row = quad_idx / total_quad_span;
	const uint quad_col = quad_idx % total_quad_span;
	const uint vertex_base = quad_col + quad_row * vert_span;

	const uvec2 indices_2[] = {
		index_offsets[which*3 + 0],
		index_offsets[which*3 + 1],
		index_offsets[which*3 + 2],
	};

	return uvec3(
		vertex_base + indices_2[0].x + indices_2[0].y * vert_span,
		vertex_base + indices_2[1].x + indices_2[1].y * vert_span,
		vertex_base + indices_2[2].x + indices_2[2].y * vert_span
	);
}


void main() {
	const uint num_threads = gl_WorkGroupSize.x;
	const uint max_vertices = gl_MeshVerticesNV.length();
	const uint max_primitives = gl_PrimitiveIndicesNV.length() / 3;

	const uint local_id = gl_LocalInvocationID.x;
	const uint patch_id = gl_WorkGroupID.x;

	const uint quad_span_per_patch = 4;
	const uint quads_per_patch = quad_span_per_patch * quad_span_per_patch;
	const uint vertices_per_patch = (quad_span_per_patch+1) * (quad_span_per_patch+1);

	const uint subdivisions = 8;
	const uint total_quad_span = 1 << subdivisions;

	const uint total_patch_span = max(total_quad_span / quads_per_patch, total_quad_span);

	const vec2 patch_size = vec2(float(quad_span_per_patch));
	const vec2 patch_offset = calculate_patch_offset(total_patch_span, patch_id) * patch_size;

	const uint num_triangles = quads_per_patch * 2;

	const uint vertex_iterations = (vertices_per_patch + num_threads - 1) / num_threads;

	for (int v = 0; v < vertex_iterations; v++) {
		uint vertex_index = min(v * num_threads + local_id, vertices_per_patch-1);

		vec2 position = (calculate_vertex(quad_span_per_patch+1, vertex_index) + patch_offset) / float(quad_span_per_patch * total_patch_span);
		vec2 position_world = (position - 0.5) * u_world_size;

		float height = texture2D(u_heightmap_sampler, position).r + float(patch_id) * 0.0;

		gl_MeshVerticesNV[vertex_index].gl_Position = u_projection_view * vec4(position_world.x, height, position_world.y, 1.0);
		vert_out[vertex_index].debug_col = vec3(float(patch_id / 200) / 200.0, float(patch_id % 200) / 200.0, 0.0);
		vert_out[vertex_index].uv = position;
	}

	const uint primitive_iterations = (num_triangles + num_threads - 1) / num_threads;

	for (int p = 0; p < primitive_iterations; p++) {
		const uint primitive_index = min(p * num_threads + local_id, num_triangles-1);
		const uvec3 triangle = calculate_indices(quad_span_per_patch, primitive_index);

		gl_PrimitiveIndicesNV[primitive_index * 3 + 0] = triangle.x;
		gl_PrimitiveIndicesNV[primitive_index * 3 + 1] = triangle.y;
		gl_PrimitiveIndicesNV[primitive_index * 3 + 2] = triangle.z;
	}

	if (local_id == 0) {
		gl_PrimitiveCountNV = num_triangles;
	}
}
