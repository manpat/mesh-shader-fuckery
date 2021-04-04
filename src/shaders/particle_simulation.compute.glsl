#version 450

#import particle
#import paint

layout(local_size_x=16, local_size_y=1, local_size_z=1) in;


layout(binding = 0) uniform sampler2D u_paint_sampler;



vec3 sample_paint(vec2 world_pos) {
	const ivec2 offsets[] = {
		ivec2(-8, 0),
		ivec2( 8, 0),
		ivec2( 0,-8),
		ivec2( 0, 8),
	};

	vec2 sample_pos = (world_pos + u_paint.world_size/2.0) / u_paint.world_size;
	vec4 samples = textureGatherOffsets(u_paint_sampler, sample_pos, offsets, 0);
	return vec3(
		samples.y - samples.x,
		samples.w - samples.z,
		texture(u_paint_sampler, sample_pos)
	);
}



void update_particle(uint particle_index) {
	Particle particle = g_particles[particle_index];

	vec3 diff = vec3(0.0, 3.0, 0.0) - particle.position;
	float dist = length(diff);

	float speed = length(particle.velocity);

	vec3 paint = sample_paint(particle.position.xz);

	vec3 attraction = (diff) * 0.2 * clamp(1.0 - dist / 10, 0.1, 3.0);
	vec3 friction = -particle.velocity * speed * 0.05 * max(3.0-abs(particle.position.y), 0.0);
	// vec3 tangent = 0.0 * attraction.zyx * vec3(-0.3, 0.1, 0.3);
	vec3 tangent = (vec3(-paint.y, 0.0, paint.x) * 5.0 + vec3(paint.x, 0.0, paint.y) * 20.0) * max(1.0 - paint.z, 0.0);
	vec3 ret = (diff + vec3(0.0, 10.0, 0.0)) * 0.1 * max(dist/10.0 - 40.0, 0.0);

	particle.velocity += (attraction + friction + tangent + ret) * 0.02;
	particle.position += particle.velocity * 0.04;

	particle.tail += (particle.position - particle.tail) * 0.04;

	g_particles[particle_index] = particle;
}


void main() {
	const uvec3 gid = gl_WorkGroupID;
	const uint global_idx = gid.x + gid.y * gl_NumWorkGroups.x + gid.z * gl_NumWorkGroups.x * gl_NumWorkGroups.y;

	const uint workgroup_size = gl_WorkGroupSize.x * gl_WorkGroupSize.y * gl_WorkGroupSize.z;

	const uint idx = gl_LocalInvocationIndex + workgroup_size * global_idx;

	if (idx < g_particles.length()) {
		update_particle(idx);
	}
}