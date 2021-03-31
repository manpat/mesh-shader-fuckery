use common::math::*;
use crate::{gl, perf};


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Particle {
	pos: Vec3,
	_0: f32,
	color: Vec3,
	_1: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Stats {
	particle_buffer_size: u32,
	max_task_output_count: u32,
}


pub struct ParticleSystem {
	rendering_program: gl::Program,
	simulation_program: gl::Program,

	particle_ssbo: gl::Buffer,
	stats_ssbo: gl::Buffer,
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

		for z in -500..500 {
			for x in -500..500 {
				for y in 0..50 {
					let x = x as f32 * 0.1;
					let z = z as f32 * 0.1;
					let y = y as f32 * 0.1;

					let pos = Vec3::new(x, y, z);
					let color = Vec3::new(1.0, 5.0, 1.0);

					particles.push(Particle {
						pos, color,
						_0: 0.0, _1: 0.0,
					});
				}
			}
		}

		let mut max_task_output_count = 0;
		unsafe {
			gl::raw::GetIntegerv(gl::raw::MAX_TASK_OUTPUT_COUNT_NV, &mut max_task_output_count);
		}

		println!("particles: {:?}", particles.len());
		println!("required tasks: {:?}", particles.len() / max_task_output_count as usize);

		let stats = Stats {
			particle_buffer_size: particles.len() as u32,
			max_task_output_count: max_task_output_count as u32,
		};

		let particle_ssbo = gl_ctx.new_buffer();
		let stats_ssbo = gl_ctx.new_buffer();

		particle_ssbo.upload(&particles, gl::BufferUsage::Static);
		stats_ssbo.upload(&[stats], gl::BufferUsage::Static);

		ParticleSystem {
			rendering_program,
			simulation_program,
			particle_ssbo,
			stats_ssbo,
		}
	}

	pub fn update(&self, gl_ctx: &gl::Context, _inst: &mut perf::Instrumenter) {

	}

	pub fn draw(&self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter) {
		gl_ctx.bind_shader_storage_buffer(0, self.particle_ssbo);
		gl_ctx.bind_shader_storage_buffer(1, self.stats_ssbo);
		gl_ctx.use_program(self.rendering_program);

		inst.start_section("particles");
		gl_ctx.draw_mesh_tasks(0, 1024);
		inst.end_section();
	}
}