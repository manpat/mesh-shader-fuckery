#version 450
#extension GL_NV_mesh_shader : require

layout(triangles) out;


struct Particle {
	vec2 position;
	vec2 size;
	vec3 color;
};

layout(std140, binding = 2) uniform UniformData {
    mat4 u_projection_view;
};

layout(std430, binding = 3) buffer ParticleData {
    Particle particles[];
};


out PerVertexData {
	vec3 color;
} vert_out[];

perprimitiveNV out PerPrimitiveData {
	vec3 color;
} prim_out[];


const vec2 positions[] = {
	vec2(-0.5, -0.5),
	vec2( 0.5, -0.5),
	vec2( 0.5,  0.5),
	vec2(-0.5,  0.5),
};

const vec3 colors[] = {
	vec3(1.0, 0.5, 1.0),
	vec3(0.5, 1.0, 1.0),
	vec3(1.0, 1.0, 0.5),
	vec3(1.0, 0.5, 0.5),
};

const uint indices[] = {0, 1, 2, 0, 2, 3};


layout(local_size_x=32) in;
layout(max_vertices=64, max_primitives=32) out;

void main() {
	uint global_id = gl_GlobalInvocationID.x;
	uint local_id = gl_LocalInvocationID.x;

	uint particle_idx = min(global_id / 2, particles.length());

	Particle particle = particles[particle_idx];
	uint quad_half = (local_id%2);

	for (int i = 0; i < 2; i++) {
		const vec2 pos = positions[quad_half*2 + i] * particle.size + particle.position;

		gl_MeshVerticesNV[local_id*2 + i].gl_Position = u_projection_view * vec4(pos, 0.0, 1.0);
		vert_out[local_id*2 + i].color = colors[quad_half*2 + i];
	}

	uint vertex_start = (local_id/2)*4;
	for (int i = 0; i < 3; i++) {
		gl_PrimitiveIndicesNV[local_id*3 + i] = vertex_start + indices[quad_half*3 + i];
	}

	prim_out[local_id].color = particle.color;
	// prim_out[local_id].color = colors[gl_WorkGroupID.x % colors.length()];

	if (local_id == 0) {
		uint work_group_base = (gl_WorkGroupID.x + 1) * gl_WorkGroupSize.x;
		int prim_diff = int(particles.length()*2) - int(work_group_base);

		if (prim_diff >= 0) {
			gl_PrimitiveCountNV = gl_WorkGroupSize.x;
		} else {
			gl_PrimitiveCountNV = gl_WorkGroupSize.x+prim_diff;
		}
	}
}


