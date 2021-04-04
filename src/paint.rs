use common::math::*;
use crate::{gl, perf};


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct PaintUniforms {
	world_size: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct BrushUniforms {
	brush_center: Vec2,
	brush_size: Vec2,
}


struct PaintOperation {
	pos: Vec2,
	size: Vec2,
}

pub struct PaintSystem {
	rendering_program: gl::Program,
	brush_program: gl::Program,

	paint_uniforms: gl::Buffer,
	brush_uniforms: gl::Buffer,
	texture: gl::Texture,

	paint_queue: Vec<PaintOperation>,
}


impl PaintSystem {
	pub fn new(gl_ctx: &gl::Context) -> PaintSystem {
		let rendering_program = gl_ctx.new_shader(&[
			(gl::raw::MESH_SHADER_NV, include_str!("shaders/paint.mesh.glsl")),
			(gl::raw::FRAGMENT_SHADER, include_str!("shaders/paint.frag.glsl")),
		]);

		let brush_program = gl_ctx.new_shader(&[
			(gl::raw::COMPUTE_SHADER, include_str!("shaders/paint_brush.compute.glsl")),
		]);

		let paint_uniforms = gl_ctx.new_buffer();
		let brush_uniforms = gl_ctx.new_buffer();
		let texture = gl_ctx.new_texture(4096, 4096, gl::raw::R32F);
		texture.clear();

		let uniforms = PaintUniforms {
			world_size: Vec2::splat(200.0),
		};

		paint_uniforms.upload(&[uniforms], gl::BufferUsage::Static);

		PaintSystem {
			rendering_program,
			brush_program,

			paint_uniforms,
			brush_uniforms,
			texture,

			paint_queue: Vec::new(),
		}
	}

	pub fn paint(&mut self, world_pos: Vec2) {
		self.paint_queue.push(PaintOperation {
			pos: world_pos,
			size: Vec2::splat(20.0),
		});
	}

	pub fn update(&mut self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter) {
		if self.paint_queue.is_empty() { return }

		inst.start_section("brush sim");

		gl_ctx.bind_image_rw(0, self.texture, gl::raw::R32F);

		gl_ctx.bind_uniform_buffer(1, self.paint_uniforms);
		gl_ctx.bind_uniform_buffer(2, self.brush_uniforms);
		gl_ctx.use_program(self.brush_program);

		for PaintOperation{pos, size} in self.paint_queue.drain(..) {
			let brush_uniforms = BrushUniforms {
				brush_center: pos,
				brush_size: size, 
			};

			self.brush_uniforms.upload(&[brush_uniforms], gl::BufferUsage::Dynamic);

			gl_ctx.dispatch_compute(256, 256, 1);
		}

		inst.end_section();

		unsafe {
			gl::raw::MemoryBarrier(gl::raw::SHADER_IMAGE_ACCESS_BARRIER_BIT);
		}
	}

	pub fn draw(&self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter) {
		unsafe {
			gl::raw::Disable(gl::raw::DEPTH_TEST);

			gl::raw::Enable(gl::raw::BLEND);
			gl::raw::BlendFunc(gl::raw::DST_COLOR, gl::raw::ONE);
			gl::raw::BlendEquation(gl::raw::FUNC_ADD);
		}
		
		gl_ctx.bind_texture(0, self.texture);
		gl_ctx.bind_uniform_buffer(1, self.paint_uniforms);
		gl_ctx.use_program(self.rendering_program);

		inst.start_section("paint");
		gl_ctx.draw_mesh_tasks(0, 1);
		inst.end_section();

		unsafe {
			gl::raw::Enable(gl::raw::DEPTH_TEST);
			gl::raw::Disable(gl::raw::BLEND);
		}
	}
}