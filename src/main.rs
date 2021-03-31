#![feature(type_ascription)]

pub mod gl;
pub mod mesh;
pub mod perf;

pub mod scene_view;
pub mod particles;

use std::error::Error;
use common::math::*;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
	projection_view: Mat4,
	up: Vec4,
	right: Vec4,
	// NOTE: align to Vec4s
}



fn init_window(sdl_video: &sdl2::VideoSubsystem) -> Result<(sdl2::video::Window, gl::Context), Box<dyn Error>> {
	let gl_attr = sdl_video.gl_attr();
	gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
	gl_attr.set_context_version(4, 5);
	gl_attr.set_context_flags().debug().set();

	let window = sdl_video.window("mesh", 700, 700)
		.position_centered()
		.resizable()
		.opengl()
		.build()?;

	let gl_ctx = window.gl_create_context()?;
	window.gl_make_current(&gl_ctx)?;

	gl::raw::load_with(|s| sdl_video.gl_get_proc_address(s) as *const _);

	assert!(sdl_video.gl_extension_supported("GL_NV_mesh_shader"));


	Ok((window, gl::Context::new(gl_ctx)))
}




fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let sdl = sdl2::init()?;
	let sdl_video = sdl.video()?;

	let (window, gl_ctx) = init_window(&sdl_video)?;

	let mut instrumenter = perf::Instrumenter::new(&gl_ctx);

	let mut uniforms = Uniforms {
		projection_view: Mat4::ident(),
		up: Vec4::from_y(1.0),
		right: Vec4::from_x(1.0),
	};

	let uniform_buffer = gl_ctx.new_buffer();
	uniform_buffer.upload(&[uniforms], gl::BufferUsage::Stream);
	gl_ctx.bind_uniform_buffer(0, uniform_buffer);

	let scene_view = scene_view::SceneView::new(&gl_ctx)?;
	let particles = particles::ParticleSystem::new(&gl_ctx);

	let mut event_pump = sdl.event_pump()?;
	let mut time = 0.0f32;
	let mut aspect = 1.0f32;

	'main: loop {
		for event in event_pump.poll_iter() {
			use sdl2::event::{Event, WindowEvent};
			use sdl2::keyboard::Keycode;

			match event {
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				Event::Window{ win_event: WindowEvent::Resized(w, h), .. } => unsafe {
					gl::raw::Viewport(0, 0, w as _, h as _);
					aspect = w as f32 / h as f32;
				}
				_ => {}
			}
		}

		time += 1.0 / 60.0;

		let camera_orientation = Mat4::yrot(time * PI / 8.0) * Mat4::xrot(-PI / 7.0);

		uniforms.up = camera_orientation * Vec4::from_y(1.0);
		uniforms.right = camera_orientation * Vec4::from_x(1.0);

		uniforms.projection_view = Mat4::perspective(PI/3.0, aspect, 0.01, 100.0)
			* Mat4::translate(Vec3::from_z(-50.0))
			* camera_orientation.inverse();

		uniform_buffer.upload(&[uniforms], gl::BufferUsage::Stream);

		particles.update(&gl_ctx, &mut instrumenter);

		unsafe {
			gl::raw::ClearColor(0.2, 0.2, 0.2, 1.0);
			gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT | gl::raw::DEPTH_BUFFER_BIT);
		}

		scene_view.draw(&gl_ctx, &mut instrumenter);
		particles.draw(&gl_ctx, &mut instrumenter);

		instrumenter.end_frame();

		window.gl_swap_window();
	}

	Ok(())
}








