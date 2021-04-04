use common::math::*;
use crate::{gl, perf, paint};

// https://onrendering.com/data/papers/isubd/isubd.pdf

pub struct Terrain {
	program: gl::Program,
}

impl Terrain {
	pub fn new(gl_ctx: &gl::Context) -> Terrain {
		let program = gl_ctx.new_shader(&[
			(gl::raw::MESH_SHADER_NV, include_str!("shaders/terrain.mesh.glsl")),
			(gl::raw::FRAGMENT_SHADER, include_str!("shaders/terrain.frag.glsl")),
		]);

		Terrain {
			program
		}
	}

	pub fn draw(&self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter, paint_resources: paint::Resources) {
		paint_resources.bind(gl_ctx, 0);

		gl_ctx.use_program(self.program);

		inst.start_section("terrain");
		gl_ctx.draw_mesh_tasks(0, 1);
		inst.end_section();
	}
}


