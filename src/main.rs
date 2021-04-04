#![feature(type_ascription)]

pub mod gl;
pub mod mesh;
pub mod perf;

pub mod scene_view;
pub mod particles;
pub mod terrain;
pub mod paint;

use std::error::Error;
use common::math::*;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
	projection_view: Mat4,
	camera_up: Vec4,
	camera_right: Vec4,
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

	let (window, mut gl_ctx) = init_window(&sdl_video)?;

	let mut instrumenter = perf::Instrumenter::new(&gl_ctx);

	gl_ctx.add_shader_import("global", include_str!("shaders/global.common.glsl"));
	gl_ctx.add_shader_import("particle", include_str!("shaders/particle.common.glsl"));
	gl_ctx.add_shader_import("paint", include_str!("shaders/paint.common.glsl"));

	let mut uniforms = Uniforms {
		projection_view: Mat4::ident(),
		camera_up: Vec4::from_y(1.0),
		camera_right: Vec4::from_x(1.0),
	};

	let uniform_buffer = gl_ctx.new_buffer();
	uniform_buffer.upload(&[uniforms], gl::BufferUsage::Stream);
	gl_ctx.bind_uniform_buffer(0, uniform_buffer);

	let scene_view = scene_view::SceneView::new(&gl_ctx)?;
	let particles = particles::ParticleSystem::new(&gl_ctx);
	let mut paint_system = paint::PaintSystem::new(&gl_ctx);
	let terrain = terrain::Terrain::new(&gl_ctx);

	let mut event_pump = sdl.event_pump()?;
	let mut aspect = 1.0f32;
	let mut zoom = 12.0f32;

	let mut yaw = 0.0f32;
	let mut pitch = -PI / 5.0;

	let mut camera_pos = Vec3::from_y(2.0);
	let mut forward_pressed = false;
	let mut back_pressed = false;
	let mut left_pressed = false;
	let mut right_pressed = false;
	let mut shift_pressed = false;

	let mut left_down = false;
	let mut right_down = false;
	let mut update_enabled = true;

	let mut wireframe_enabled = false;

	let mut scene_view_enabled = true;
	let mut particles_enabled = true;
	let mut paint_enabled = true;
	let mut terrain_enabled = true;

	let mut mouse_world_pos = Vec2::zero();

	'main: loop {
		for event in event_pump.poll_iter() {
			use sdl2::event::{Event, WindowEvent};
			use sdl2::keyboard::Keycode;
			use sdl2::mouse::MouseButton;

			match event {
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				Event::Window{ win_event: WindowEvent::Resized(w, h), .. } => unsafe {
					gl::raw::Viewport(0, 0, w as _, h as _);
					aspect = w as f32 / h as f32;
				}

				Event::MouseWheel { y, .. } => {
					zoom = (zoom.log2() - y as f32 / 5.0).exp2();
				}

				Event::MouseMotion { xrel, yrel, x, y, .. } => {
					if left_down {
						yaw += xrel as f32 * 0.005;
						pitch = (pitch - yrel as f32 * 0.005).clamp(-PI, PI);
					}

					let (w, h) = window.drawable_size();
					let mouse_x =  x as f32 / w as f32 * 2.0 - 1.0;
					let mouse_y = -(y as f32 / h as f32 * 2.0 - 1.0);

					let proj_view_inv = uniforms.projection_view.inverse();

					let near_point = proj_view_inv * Vec4::new(mouse_x, mouse_y, -1.0, 1.0);
					let near_point = near_point.to_vec3() / near_point.w;

					let far_point = proj_view_inv * Vec4::new(mouse_x, mouse_y, 1.0, 1.0);
					let far_point = far_point.to_vec3() / far_point.w;

					let ray_dir = (far_point - near_point).normalize();

					let plane = Plane::new(Vec3::from_y(1.0), 0.0);

					if plane.normal.dot(ray_dir).abs() > 0.01 {
						let t = (plane.length - plane.normal.dot(near_point)) / plane.normal.dot(ray_dir);
						let world_pos = near_point + ray_dir * t;

						mouse_world_pos = world_pos.to_xz();
					}
				}

				Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
					MouseButton::Left => { left_down = true }
					MouseButton::Right => { right_down = true }
					_ => {}
				}

				Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
					MouseButton::Left => { left_down = false }
					MouseButton::Right => { right_down = false }
					_ => {}
				}

				Event::KeyDown { keycode: Some(Keycode::Space), .. } => { update_enabled = !update_enabled }
				Event::KeyDown { keycode: Some(keycode), .. } => match keycode {
					Keycode::Num1 => { scene_view_enabled = !scene_view_enabled }
					Keycode::Num2 => { particles_enabled = !particles_enabled }
					Keycode::Num3 => { paint_enabled = !paint_enabled }
					Keycode::Num4 => { terrain_enabled = !terrain_enabled }

					Keycode::Z => {
						wireframe_enabled = !wireframe_enabled;
						gl_ctx.set_wireframe(wireframe_enabled);
					}

					Keycode::W => { forward_pressed = true }
					Keycode::S => { back_pressed = true }
					Keycode::A => { left_pressed = true }
					Keycode::D => { right_pressed = true }
					Keycode::LShift => { shift_pressed = true }
					_ => {}
				}
				Event::KeyUp { keycode: Some(keycode), .. } => match keycode {
					Keycode::W => { forward_pressed = false }
					Keycode::S => { back_pressed = false }
					Keycode::A => { left_pressed = false }
					Keycode::D => { right_pressed = false }
					Keycode::LShift => { shift_pressed = false }
					_ => {}
				}
				_ => {}
			}
		}

		if right_down {
			paint_system.paint(mouse_world_pos);
		}

		let camera_yaw_mat = Mat4::yrot(yaw);
		let camera_orientation = camera_yaw_mat * Mat4::xrot(pitch);

		let move_speed = match shift_pressed {
			true => 15.0,
			false => 5.0,
		};

		let cam_move_fwd = camera_yaw_mat * Vec3::from_z(-move_speed / 60.0);
		let cam_move_right = camera_yaw_mat * Vec3::from_x(move_speed / 60.0);

		if forward_pressed { camera_pos += cam_move_fwd }
		if back_pressed { camera_pos -= cam_move_fwd }
		if left_pressed { camera_pos -= cam_move_right }
		if right_pressed { camera_pos += cam_move_right }


		uniforms.camera_up = camera_orientation * Vec4::from_y(1.0);
		uniforms.camera_right = camera_orientation * Vec4::from_x(1.0);

		uniforms.projection_view = Mat4::perspective(PI/3.0, aspect, 0.1, 1000.0)
			* Mat4::translate(Vec3::from_z(-zoom))
			* camera_orientation.inverse()
			* Mat4::translate(-camera_pos);

		uniform_buffer.upload(&[uniforms], gl::BufferUsage::Stream);

		if update_enabled {
			if particles_enabled {
				particles.update(&gl_ctx, &mut instrumenter, paint_system.resources());
			}

			if paint_enabled {
				paint_system.update(&gl_ctx, &mut instrumenter);
			}
		}

		unsafe {
			gl::raw::ClearColor(1.0, 1.0, 1.0, 1.0);
			gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT | gl::raw::DEPTH_BUFFER_BIT);
		}

		if scene_view_enabled {
			scene_view.draw(&gl_ctx, &mut instrumenter);
		}

		if paint_enabled {
			paint_system.draw(&gl_ctx, &mut instrumenter);
		}

		if particles_enabled {
			particles.draw(&gl_ctx, &mut instrumenter);
		}

		if terrain_enabled {
			terrain.draw(&gl_ctx, &mut instrumenter);
		}

		instrumenter.end_frame();

		window.gl_swap_window();
	}

	Ok(())
}








