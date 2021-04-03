use common::math::*;
use crate::{gl, perf};


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Particle {
	pos: Vec3,
	_0: f32,
	velocity: Vec3,
	_1: f32,
	tail: Vec3,
	_2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct StatsBuffer {
	particle_buffer_size: u32,
	max_task_output_count: u32,
}


pub struct ParticleSystem {
	rendering_program: gl::Program,
	simulation_program: gl::Program,

	particle_ssbo: gl::Buffer,
	stats_ssbo: gl::Buffer,

	particle_buffer_size: u32,
	max_task_output_count: u32,
}

impl ParticleSystem {
	pub fn new(gl_ctx: &gl::Context) -> ParticleSystem {
		let rendering_program = gl_ctx.new_shader(&[
			(gl::raw::TASK_SHADER_NV, include_str!("particle.task.glsl")),
			(gl::raw::MESH_SHADER_NV, include_str!("particle.mesh.glsl")),
			(gl::raw::FRAGMENT_SHADER, include_str!("particle.frag.glsl")),
		]);

		let simulation_program = gl_ctx.new_shader(&[
			(gl::raw::COMPUTE_SHADER, include_str!("particle_simulation.compute.glsl")),
		]);

		
		let mut particles = Vec::new();

		let mut offset = 0.0f32;

		for z in -10..10 {
			for x in -10..10 {
				for y in 0..64 {
					let x = x as f32 * 0.1;
					let z = z as f32 * 0.1;
					let y = y as f32 * 0.1;

					let pos = Vec3::new(x, y, z);
					let velocity = Vec3::from_y_angle(z * y + x * x + offset)
						+ Vec3::from_x_angle(x * z - y * offset);

					let velocity = velocity * (offset * offset).sin() * 0.1;

					let tail = pos;

					particles.push(Particle {
						pos, velocity, tail,
						_0: 0.0, _1: 0.0, _2: 0.0,
					});

					offset += 0.5;
				}
			}
		}

		let mut max_task_output_count = 0;
		unsafe {
			gl::raw::GetIntegerv(gl::raw::MAX_TASK_OUTPUT_COUNT_NV, &mut max_task_output_count);
		}

		println!("particles: {:?}", particles.len());
		println!("required tasks: {:?}", particles.len() / max_task_output_count as usize);

		let particle_buffer_size = particles.len() as u32;
		let max_task_output_count = max_task_output_count as u32;

		let stats = StatsBuffer {particle_buffer_size, max_task_output_count};

		let particle_ssbo = gl_ctx.new_buffer();
		let stats_ssbo = gl_ctx.new_buffer();

		particle_ssbo.upload(&particles, gl::BufferUsage::Static);
		stats_ssbo.upload(&[stats], gl::BufferUsage::Static);

		ParticleSystem {
			rendering_program,
			simulation_program,
			particle_ssbo,
			stats_ssbo,

			particle_buffer_size,
			max_task_output_count,
		}
	}

	pub fn update(&self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter) {
		gl_ctx.bind_shader_storage_buffer(0, self.particle_ssbo);
		gl_ctx.use_program(self.simulation_program);

		let particles_per_invocation = 16;

		inst.start_section("particles sim");
		gl_ctx.dispatch_compute((self.particle_buffer_size + particles_per_invocation - 1) / particles_per_invocation, 1, 1);
		inst.end_section();

		unsafe {
			gl::raw::MemoryBarrier(gl::raw::SHADER_STORAGE_BARRIER_BIT);
		}
	}

	pub fn draw(&self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter) {
		unsafe {
			gl::raw::Enable(gl::raw::BLEND);
			gl::raw::BlendFunc(gl::raw::DST_COLOR, gl::raw::ZERO);
			gl::raw::BlendEquation(gl::raw::FUNC_ADD);

			gl::raw::DepthMask(0);
		}
		
		gl_ctx.bind_shader_storage_buffer(0, self.particle_ssbo);
		gl_ctx.bind_shader_storage_buffer(1, self.stats_ssbo);
		gl_ctx.use_program(self.rendering_program);

		let num_task_invocations = (self.particle_buffer_size + self.max_task_output_count - 1) / self.max_task_output_count;

		inst.start_section("particles");
		gl_ctx.draw_mesh_tasks(0, num_task_invocations);
		inst.end_section();

		unsafe {
			gl::raw::Disable(gl::raw::BLEND);
			gl::raw::DepthMask(1);
		}
	}
}