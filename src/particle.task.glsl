#version 450
#extension GL_NV_mesh_shader : require

layout(local_size_x=1) in;

layout(std430, binding = 1) buffer Stats {
    uint particle_buffer_size;
	uint max_task_output_count;
};

taskNV out Task {
  uint t_base_particle_id;
  uint t_global_id;
};

void main() {
	const uint global_id = gl_WorkGroupID.x;

	const uint particles_per_task = 16;
	const uint max_particles = particle_buffer_size / particles_per_task;

	const uint already_spawned = max_task_output_count * global_id;

	if (already_spawned < max_particles) {
		const uint num_particles = max_particles - already_spawned;
		gl_TaskCountNV = clamp(num_particles, 0, max_task_output_count);
	} else {
		gl_TaskCountNV = 0;
	}


	t_global_id = global_id;
	t_base_particle_id = global_id * max_task_output_count;
}