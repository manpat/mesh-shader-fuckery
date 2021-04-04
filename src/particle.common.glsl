

struct Particle {
	vec3 position;
	vec3 velocity;
	vec3 tail;
};


layout(std430, binding = 0) buffer ParticleData {
    Particle g_particles[];
};

