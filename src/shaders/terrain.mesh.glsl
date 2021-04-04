#version 450
#extension GL_NV_mesh_shader : require

#import global

layout(triangles) out;
layout(local_size_x=2) in;
layout(max_vertices=4, max_primitives=2) out;


layout(binding = 0) uniform sampler2D u_heightmap_sampler;

out PerVertexData {
	vec2 uv;
} vert_out[];


const vec2 positions[] = {
	vec2( 0.0, 0.0),
	vec2( 1.0, 0.0),
	vec2( 1.0, 1.0),
	vec2( 0.0, 1.0),
};

const uint indices[] = {0, 1, 2,  0, 2, 3};


void main() {
	const uint num_threads = gl_WorkGroupSize.x;
	const uint local_id = gl_LocalInvocationID.x;

	for (int v = 0; v < 2; v++) {
		uint vertex_index = v * num_threads + local_id;

		vec2 position = positions[vertex_index % positions.length()];
		vec2 position_world = (position - 0.5) * u_world_size;

		gl_MeshVerticesNV[vertex_index].gl_Position = u_projection_view * vec4(position_world.x, 0.0, position_world.y, 1.0);
		vert_out[vertex_index].uv = position;
	}

	const uint which = local_id % 2;
	gl_PrimitiveIndicesNV[local_id * 3 + 0] = indices[which * 3 + 0];
	gl_PrimitiveIndicesNV[local_id * 3 + 1] = indices[which * 3 + 1];
	gl_PrimitiveIndicesNV[local_id * 3 + 2] = indices[which * 3 + 2];

	if (local_id == 0) {
		gl_PrimitiveCountNV = 2;
	}
}
