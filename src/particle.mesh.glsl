#version 450
#extension GL_NV_mesh_shader : require

layout(triangles) out;


layout(local_size_x=32) in;
layout(max_vertices=64, max_primitives=32) out;


struct Particle {
	vec3 position;
	vec3 color;
};


taskNV in Task {
  uint t_base_particle_id;
  uint t_global_id;
};

layout(std140, binding = 0) uniform UniformData {
    layout(row_major) mat4 u_projection_view;
	vec4 u_up;
	vec4 u_right;
};

layout(std430, binding = 0) buffer ParticleData {
    Particle particles[];
};


// out PerVertexData {
// 	vec3 color;
// } vert_out[];

perprimitiveNV out PerPrimitiveData {
	vec3 color;
} prim_out[];


const vec2 positions[] = {
	vec2(-0.5, -0.5),
	vec2( 0.5, -0.5),
	vec2( 0.5,  0.5),
	vec2(-0.5,  0.5),
};

const uint indices[] = {0, 1, 2, 0, 2, 3};



void main() {
	const uint num_threads = gl_WorkGroupSize.x;
	const uint max_vertices = gl_MeshVerticesNV.length();
	const uint max_primitives = gl_PrimitiveIndicesNV.length() / 3;

	const uint primitives_per_particle = indices.length() / 3;

	const uint local_id = gl_LocalInvocationID.x;

	const uint particle_count = particles.length();
	const uint primitive_count_real = particle_count * 2;
	const uint primitive_count = 32; // min(primitive_count_real, max_primitives);

	const uint vertex_iterations = (max_vertices + num_threads - 1) / num_threads;

	const uint work_group_base = (t_base_particle_id + gl_WorkGroupID.x) * num_threads;

	for (int v = 0; v < vertex_iterations; v++) {
		uint vertex_index = min(v * num_threads + local_id, max_vertices-1);

		Particle particle = particles[(work_group_base * vertex_iterations + vertex_index) / positions.length()];
		vec2 pos_uv = positions[vertex_index % positions.length()];
		vec3 pos_local = pos_uv.x * u_right.xyz + pos_uv.y * u_up.xyz;

		vec3 pos = particle.position + pos_local * 0.05;

		gl_MeshVerticesNV[vertex_index].gl_Position = u_projection_view * vec4(pos, 1.0);
	}

	const uint primitive_iterations = (max_primitives + num_threads - 1) / num_threads;

	for (int p = 0; p < primitive_iterations; p++) {
		const uint primitive_index = min(p * num_threads + local_id, max_primitives-1);
		const uint primitive_base = primitive_index / primitives_per_particle * positions.length();
		const uint which = primitive_index % 2;

		gl_PrimitiveIndicesNV[primitive_index * 3 + 0] = primitive_base + indices[which*3 + 0];
		gl_PrimitiveIndicesNV[primitive_index * 3 + 1] = primitive_base + indices[which*3 + 1];
		gl_PrimitiveIndicesNV[primitive_index * 3 + 2] = primitive_base + indices[which*3 + 2];

		// prim_out[primitive_index].color = fract(vec3(uvec3(gl_WorkGroupID.x / 2, gl_WorkGroupID.x / 4, gl_WorkGroupID.x / 8)) / 5.0 ); // colors[meshlet_id % colors.length()];
		prim_out[primitive_index].color = fract(vec3(uvec3(t_global_id, t_global_id / 5, t_global_id / 25)) / 5.0 ); // colors[meshlet_id % colors.length()];
	}

	if (local_id == 0) {
		// gl_PrimitiveCountNV = meshlet.primitive_count;
		gl_PrimitiveCountNV = primitive_count;
	}
}
