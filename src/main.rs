#![feature(type_ascription)]

pub mod gl;
pub mod mesh;

use std::error::Error;
use common::math::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Particle {
	pos: Vec2,
	size: Vec2,
	color: Vec3,
	_pad: f32,
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Uniforms {
	projection_view: Mat4,
	// NOTE: align to Vec4s
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
	pos: Vec3, _1: f32,
	color: Vec3, _2: f32,
}

impl Vertex {
	fn new(pos: Vec3, color: Vec3) -> Vertex {
		Vertex { pos, color, _1: 0.0, _2: 0.0 }
	}
}




fn init_window(sdl_video: &sdl2::VideoSubsystem) -> Result<(sdl2::video::Window, gl::GlContext), Box<dyn Error>> {
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


	Ok((window, gl::GlContext::new(gl_ctx)))
}




fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let sdl = sdl2::init()?;
	let sdl_video = sdl.video()?;

	let (window, gl_ctx) = init_window(&sdl_video)?;

	let toy_project = toy::load(include_bytes!("fish.toy"))?;
	let toy_scene = toy_project.find_scene("main").expect("Missing main scene");

	let mut mb = mesh::MeshletBuilder::new();

	for entity in toy_scene.entities() {
		if entity.name.contains('_') { continue }

		let mesh_data = match entity.mesh_data() {
			Some(md) => md,
			None => continue,
		};

		let color_data = mesh_data.color_data(None).unwrap();

		let transform = entity.transform();
		let vertices = mesh_data.positions.iter()
			.zip(&color_data.data)
			.map(|(&pos, &col)| {
				Vertex::new(transform * pos, col.to_vec3())
			})
			.collect(): Vec<_>;

		mb.append(&vertices, &mesh_data.indices);
	}

	let mesh = mb.build();

	println!("verts: {}", mesh.vertex_data.len());
	println!("meshlets: {}", mesh.num_meshlets);


	let mut uniforms = Uniforms {
		projection_view: Mat4::ident(),
	};


	let uniform_buffer = gl_ctx.new_buffer();
	let vertex_ssbo = gl_ctx.new_buffer();
	let meshlet_data_ssbo = gl_ctx.new_buffer();

	uniform_buffer.upload(&[uniforms], gl::BufferUsage::Stream);
	vertex_ssbo.upload(&mesh.vertex_data, gl::BufferUsage::Static);
	meshlet_data_ssbo.upload(&mesh.meshlet_data, gl::BufferUsage::Static);

	gl_ctx.bind_uniform_buffer(2, uniform_buffer);
	gl_ctx.bind_shader_storage_buffer(3, vertex_ssbo);
	gl_ctx.bind_shader_storage_buffer(4, meshlet_data_ssbo);

	let timer_object = unsafe {
		let mut id = 0;
		gl::raw::GenQueries(1, &mut id);
		id
	};

	let mut timer_waiting = false;
	let mut timer_avg = 0.0f64;


	let main_program = gl_ctx.new_shader(&[
		(gl::raw::MESH_SHADER_NV, include_str!("main.mesh.glsl")),
		(gl::raw::FRAGMENT_SHADER, include_str!("particle.frag.glsl")),
	]);

	let mut event_pump = sdl.event_pump()?;
	let mut time = 0.0f32;

	'main: loop {
		for event in event_pump.poll_iter() {
			use sdl2::event::Event;
			use sdl2::keyboard::Keycode;

			match event {
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
				_ => {}
			}
		}

		time += 1.0 / 60.0;

		uniforms.projection_view = Mat4::perspective(PI/3.0, 1.0, 0.01, 100.0)
			* Mat4::translate(Vec3::from_z(-10.0))
			* Mat4::xrot(PI / 7.0)
			* Mat4::yrot(-time * PI / 8.0)
			;

		uniform_buffer.upload(&[uniforms], gl::BufferUsage::Stream);

		unsafe {
			gl::raw::ClearColor(0.2, 0.2, 0.2, 1.0);
			gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT | gl::raw::DEPTH_BUFFER_BIT);

			gl_ctx.use_program(main_program);

			if !timer_waiting {
				gl::raw::BeginQuery(gl::raw::TIME_ELAPSED, timer_object);
			}

			gl::raw::DrawMeshTasksNV(0, mesh.num_meshlets as _);

			if !timer_waiting {
				gl::raw::EndQuery(gl::raw::TIME_ELAPSED);
				timer_waiting = true;
			}


			if timer_waiting {
				let mut ready = 0;
				gl::raw::GetQueryObjectiv(timer_object, gl::raw::QUERY_RESULT_AVAILABLE, &mut ready);
				if ready != 0 {
					let mut time_elapsed = 0;
					gl::raw::GetQueryObjectiv(timer_object, gl::raw::QUERY_RESULT, &mut time_elapsed);

					timer_avg += (time_elapsed as f64 - timer_avg) * 0.1;
					println!("timer_avg {:.2}us", timer_avg / 1000.0);

					timer_waiting = false;
				}
			};
		}

		window.gl_swap_window();
	}

	Ok(())
}








