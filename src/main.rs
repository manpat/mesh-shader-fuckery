#![feature(type_ascription)]

pub mod gl {
	include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

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




fn main() -> Result<(), Box<dyn Error>> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let sdl = sdl2::init()?;
	let sdl_video = sdl.video()?;

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

	gl::load_with(|s| sdl_video.gl_get_proc_address(s) as *const _);

	assert!(sdl_video.gl_extension_supported("GL_NV_mesh_shader"));


	unsafe {
		let mut vao = 0;
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);

		gl::DebugMessageCallback(Some(gl_message_callback), std::ptr::null());
		gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

		gl::Enable(gl::DEPTH_TEST);

		// Disable performance messages
		gl::DebugMessageControl(
			gl::DONT_CARE,
			gl::DEBUG_TYPE_PERFORMANCE,
			gl::DONT_CARE,
			0, std::ptr::null(),
			0 // false
		);

		// Disable notification messages
		gl::DebugMessageControl(
			gl::DONT_CARE,
			gl::DONT_CARE,
			gl::DEBUG_SEVERITY_NOTIFICATION,
			0, std::ptr::null(),
			0 // false
		);
	}


	let toy_project = toy::load(include_bytes!("fish.toy"))?;
	let toy_scene = toy_project.find_scene("main").expect("Missing main scene");

	let mut mb = MeshletBuilder::new();

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



	// let vertices = [
	// 	Vertex::new(Vec3::new(-1.0,-1.0,-1.0), Vec3::splat(1.0)),
	// 	Vertex::new(Vec3::new(-1.0,-1.0, 1.0), Vec3::splat(1.0)),
	// 	Vertex::new(Vec3::new( 1.0,-1.0, 1.0), Vec3::splat(1.0)),
	// 	Vertex::new(Vec3::new( 1.0,-1.0,-1.0), Vec3::splat(1.0)),

	// 	Vertex::new(Vec3::new(-1.0, 1.0,-1.0), Vec3::splat(0.5)),
	// 	Vertex::new(Vec3::new( 1.0, 1.0,-1.0), Vec3::splat(0.5)),
	// 	Vertex::new(Vec3::new( 1.0, 1.0, 1.0), Vec3::splat(0.5)),
	// 	Vertex::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::splat(0.5)),
	// ];

	// let indices = [
	// 	0, 1, 2,  0, 2, 3, // bottom
	// 	4, 5, 6,  4, 7, 6, // top

	// 	0, 1, 7,  0, 7, 4, // left
	// 	3, 6, 2,  3, 5, 6, // right

	// 	0, 3, 5,  0, 5, 4, // back
	// 	1, 6, 7,  1, 2, 6, // front
	// ];


	// let meshlet_data = generate_meshlet_data(&indices);
	// dbg!(&mesh);


	// let particle_program = {
	// 	let mesh_sh = compile_shader(include_str!("particle.mesh.glsl"), gl::MESH_SHADER_NV);
	// 	let frag_sh = compile_shader(include_str!("particle.frag.glsl"), gl::FRAGMENT_SHADER);

	// 	link_shaders(&[mesh_sh, frag_sh])
	// };


	// let mut data = Vec::new();

	// for y in -30..30 {
	// 	for x in -30..11 {
	// 		data.push(Particle {
	// 			pos: Vec2::new((x as f32 + 0.5) / 30.0, (y as f32 + 0.5) / 30.0),
	// 			size: Vec2::splat(0.03),
	// 			color: Vec3::new(1.0, 0.4, 1.0),
	// 			_pad: 0.0,
	// 		})
	// 	}
	// }

	// data.push(Particle {
	// 	pos: Vec2::new(0.5, 0.0),
	// 	size: Vec2::splat(0.1),
	// 	color: Vec3::new(1.0, 0.4, 1.0),
	// 	_pad: 0.0,
	// });

	// data.push(Particle {
	// 	pos: Vec2::new(0.5, 0.2),
	// 	size: Vec2::splat(0.1),
	// 	color: Vec3::new(0.4, 1.0, 1.0),
	// 	_pad: 0.0,
	// });

	// data.push(Particle {
	// 	pos: Vec2::new(0.5, 0.4),
	// 	size: Vec2::splat(0.1),
	// 	color: Vec3::new(1.0, 1.0, 0.4),
	// 	_pad: 0.0,
	// });

	// let _particle_ssbo = unsafe {
	// 	let mut buf = 0;
	// 	gl::GenBuffers(1, &mut buf);
	// 	gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buf);
	// 	gl::BufferData(
	// 		gl::SHADER_STORAGE_BUFFER,
	// 		(data.len() * std::mem::size_of::<Particle>()) as _,
	// 		data.as_ptr() as *const _,
	// 		gl::STATIC_DRAW
	// 	);

	// 	gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, buf);
	// 	gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
	// 	buf
	// };

	let mut uniforms = Uniforms {
		projection_view: Mat4::ident(),
	};


	let uniform_buffer = unsafe {
		let mut buf = 0;
		gl::GenBuffers(1, &mut buf);
		gl::BindBuffer(gl::UNIFORM_BUFFER, buf);
		gl::BufferData(
			gl::UNIFORM_BUFFER,
			std::mem::size_of_val(&uniforms) as _,
			&uniforms as *const _ as *const _,
			gl::STREAM_DRAW
		);

		gl::BindBufferBase(gl::UNIFORM_BUFFER, 2, buf);
		gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
		buf
	};

	let _vertex_ssbo = unsafe {
		let mut buf = 0;
		gl::GenBuffers(1, &mut buf);
		gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buf);
		gl::BufferData(
			gl::SHADER_STORAGE_BUFFER,
			(mesh.vertex_data.len() * std::mem::size_of::<Vertex>()) as _,
			mesh.vertex_data.as_ptr() as *const _,
			gl::STATIC_DRAW
		);

		gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, buf);
		gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
		buf
	};

	let _meshlet_data_ssbo = unsafe {
		let mut buf = 0;
		gl::GenBuffers(1, &mut buf);
		gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buf);
		gl::BufferData(
			gl::SHADER_STORAGE_BUFFER,
			mesh.meshlet_data.len() as _,
			mesh.meshlet_data.as_ptr() as *const _,
			gl::STATIC_DRAW
		);

		gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, buf);
		gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
		buf
	};


	let timer_object = unsafe {
		let mut id = 0;
		gl::GenQueries(1, &mut id);
		id
	};

	let mut timer_waiting = false;
	let mut timer_avg = 0.0f64;


	let main_program = {
		let mesh_sh = compile_shader(include_str!("main.mesh.glsl"), gl::MESH_SHADER_NV);
		let frag_sh = compile_shader(include_str!("particle.frag.glsl"), gl::FRAGMENT_SHADER);

		link_shaders(&[mesh_sh, frag_sh])
	};

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

		unsafe {
			gl::BindBuffer(gl::UNIFORM_BUFFER, uniform_buffer);
			gl::BufferData(
				gl::UNIFORM_BUFFER,
				std::mem::size_of_val(&uniforms) as _,
				&uniforms as *const _ as *const _,
				gl::STREAM_DRAW
			);
			gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
		}

		unsafe {
			gl::ClearColor(0.2, 0.2, 0.2, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

			gl::UseProgram(main_program);

			if !timer_waiting {
				gl::BeginQuery(gl::TIME_ELAPSED, timer_object);
			}

			gl::DrawMeshTasksNV(0, mesh.num_meshlets as _);

			if !timer_waiting {
				gl::EndQuery(gl::TIME_ELAPSED);
				timer_waiting = true;
			}


			if timer_waiting {
				let mut ready = 0;
				gl::GetQueryObjectiv(timer_object, gl::QUERY_RESULT_AVAILABLE, &mut ready);
				if ready != 0 {
					let mut time_elapsed = 0;
					gl::GetQueryObjectiv(timer_object, gl::QUERY_RESULT, &mut time_elapsed);

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


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MeshletDescriptor {
	vertex_count: u32,
	primitive_count: u32,

	/// offset into vertex_indices
	vertex_begin: u32,

	/// offset into primitive_indices
	primitive_begin: u32,
}

#[derive(Debug)]
struct MeshData {
	vertex_data: Vec<Vertex>,
	meshlet_data: Vec<u8>,
	num_meshlets: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MeshletDataHeader {
	vertex_indices_offset: u32,
	primitive_indices_offset: u32,
}


const MAX_MESHLET_TRIANGLES: usize = 126;
const MAX_MESHLET_VERTICES: usize = 64;
// const MAX_MESHLET_TRIANGLES: usize = 6;
// const MAX_MESHLET_VERTICES: usize = 5;


struct MeshletBuilder {
	vertices: Vec<Vertex>,

	meshlet_descriptors: Vec<MeshletDescriptor>,
	vertex_indices: Vec<u32>,
	primitive_indices: Vec<u8>,

	vertex_begin: usize,
	primitive_begin: usize,
}

impl MeshletBuilder {
	fn new() -> MeshletBuilder {
		MeshletBuilder {
			vertices: Vec::new(),

			meshlet_descriptors: Vec::new(),
			vertex_indices: Vec::new(),
			primitive_indices: Vec::new(),

			vertex_begin: 0,
			primitive_begin: 0,
		}
	}

	fn append(&mut self, vertices: &[Vertex], triangle_indices: &[u16]) {
		let vertex_start = self.vertices.len() as u32;

		self.vertices.extend_from_slice(vertices);

		for triangle in triangle_indices.chunks(3) {
			let mut vertex_unique = [false; 3];
			for (unique, &vertex) in vertex_unique.iter_mut().zip(triangle) {
				let vertex = vertex_start + vertex as u32;
				*unique = !self.vertex_indices[self.vertex_begin..].contains(&vertex);
			}

			let new_vertices = vertex_unique.iter().filter(|v| **v).count();
			let vertex_count = self.vertex_indices.len() - self.vertex_begin;
			let primitive_count = (self.primitive_indices.len() - self.primitive_begin) / 3;

			if vertex_count + new_vertices > MAX_MESHLET_VERTICES {
				self.meshlet_descriptors.push(MeshletDescriptor {
					vertex_count: vertex_count as u32,
					primitive_count: primitive_count as u32,
					vertex_begin: self.vertex_begin as u32,
					primitive_begin: (self.primitive_begin / 3) as u32,
				});

				self.primitive_begin = self.primitive_indices.len();
				self.vertex_begin = self.vertex_indices.len();

				vertex_unique = [true; 3];
			}

			for (&vertex, &unique) in triangle.iter().zip(&vertex_unique) {
				if unique {
					self.vertex_indices.push(vertex_start + vertex as u32);
				}
			}

			let vertices = &self.vertex_indices[self.vertex_begin..];
			for &vertex in triangle {
				let vertex = vertex_start + vertex as u32;
				let prim_index = vertices.iter().position(|&v| v == vertex).unwrap() as u8;
				self.primitive_indices.push(prim_index);

			}

			let primitive_count = (self.primitive_indices.len() - self.primitive_begin) / 3;
			let vertex_count = self.vertex_indices.len() - self.vertex_begin;

			if primitive_count >= MAX_MESHLET_TRIANGLES {
				self.meshlet_descriptors.push(MeshletDescriptor {
					vertex_count: vertex_count as u32,
					primitive_count: primitive_count as u32,
					vertex_begin: self.vertex_begin as u32,
					primitive_begin: (self.primitive_begin / 3) as u32,
				});

				self.primitive_begin = self.primitive_indices.len();
				self.vertex_begin = self.vertex_indices.len();
			}
		}
	}


	fn build(mut self) -> MeshData {
		let primitive_count = (self.primitive_indices.len() - self.primitive_begin) / 3;
		let vertex_count = self.vertex_indices.len() - self.vertex_begin;

		if primitive_count > 0 {
			self.meshlet_descriptors.push(MeshletDescriptor {
				vertex_count: vertex_count as u32,
				primitive_count: primitive_count as u32,
				vertex_begin: self.vertex_begin as u32,
				primitive_begin: (self.primitive_begin / 3) as u32,
			});
		}

		// pad to 32b
		// if self.vertex_indices.len() % 2 == 1 {
		// 	self.vertex_indices.push(0);
		// }

		// pad to 32b
		for _ in self.primitive_indices.len() .. (self.primitive_indices.len() + 3) / 4*4 {
			self.primitive_indices.push(0);
		}


		use std::mem::size_of;

		let header_size = size_of::<MeshletDataHeader>();
		let meshlet_descriptor_size = size_of::<MeshletDescriptor>() * self.meshlet_descriptors.len();
		let vertex_indices_size = size_of::<u32>() * self.vertex_indices.len();
		let primitive_indices_size = size_of::<u8>() * self.primitive_indices.len();

		assert!(header_size % 4 == 0);
		assert!(meshlet_descriptor_size % 4 == 0);
		assert!(vertex_indices_size % 4 == 0);
		assert!(primitive_indices_size % 4 == 0);

		let header = MeshletDataHeader {
			vertex_indices_offset: ((header_size + meshlet_descriptor_size) / 4) as _,
			primitive_indices_offset: ((header_size + meshlet_descriptor_size + vertex_indices_size) / 4) as _,
		};

		let buffer_size = header_size
			+ meshlet_descriptor_size
			+ vertex_indices_size
			+ primitive_indices_size;

		let mut buffer = vec![0u8; buffer_size];

		{
			let (header_bytes, rest) = buffer.split_at_mut(header_size);
			let (meshlet_desc_bytes, rest) = rest.split_at_mut(meshlet_descriptor_size);
			let (vertex_indices_bytes, rest) = rest.split_at_mut(vertex_indices_size);
			let (primitve_indices_bytes, _) = rest.split_at_mut(primitive_indices_size);

			header_bytes.copy_from_slice(as_bytes(&[header]));
			meshlet_desc_bytes.copy_from_slice(as_bytes(&self.meshlet_descriptors));
			vertex_indices_bytes.copy_from_slice(as_bytes(&self.vertex_indices));
			primitve_indices_bytes.copy_from_slice(as_bytes(&self.primitive_indices));
		}

		MeshData {
			vertex_data: self.vertices,
			meshlet_data: buffer,
			num_meshlets: self.meshlet_descriptors.len(),
		}
	}

}

fn as_bytes<T>(buf: &[T]) -> &[u8] {
	use std::mem::size_of;
	unsafe {
		std::slice::from_raw_parts(
			buf.as_ptr() as *const u8,
			buf.len() * size_of::<T>()
		)
	}
}






fn compile_shader(src: &str, ty: u32) -> u32 {
	use std::ffi::CString;
	use std::str;

	unsafe {
		let handle = gl::CreateShader(ty);
		let src = CString::new(src.as_bytes()).unwrap();

		gl::ShaderSource(handle, 1, &src.as_ptr(), std::ptr::null());
		gl::CompileShader(handle);

		let mut status = 0;
		gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);

		if status == 0 {
			let mut length = 0;
			gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut length);

			let mut buffer = vec![0u8; length as usize];
			gl::GetShaderInfoLog(
				handle,
				length,
				std::ptr::null_mut(),
				buffer.as_mut_ptr() as *mut _
			);

			let error = str::from_utf8(&buffer[..buffer.len()-1]).unwrap();

			panic!("Shader compile failed!\n{}", error);
		}

		handle
	}
}

fn link_shaders(shaders: &[u32]) -> u32 {
	unsafe {
		let handle = gl::CreateProgram();
		for &sh in shaders {
			gl::AttachShader(handle, sh);
		}

		gl::LinkProgram(handle);

		let mut status = 0;
		gl::GetProgramiv(handle, gl::LINK_STATUS, &mut status);

		if status == 0 {
			let mut buf = [0u8; 1024];
			let mut len = 0;
			gl::GetProgramInfoLog(handle, buf.len() as _, &mut len, buf.as_mut_ptr() as _);

			panic!("shader link failed: {}", std::str::from_utf8(&buf[..len as usize]).unwrap());
		}

		for &sh in shaders {
			gl::DeleteShader(sh);
		}

		handle
	}
}



extern "system" fn gl_message_callback(source: u32, ty: u32, _id: u32, severity: u32,
	_length: i32, msg: *const i8, _ud: *mut std::ffi::c_void)
{
	let severity = match severity {
		gl::DEBUG_SEVERITY_LOW => "low",
		gl::DEBUG_SEVERITY_MEDIUM => "medium",
		gl::DEBUG_SEVERITY_HIGH => "high",
		gl::DEBUG_SEVERITY_NOTIFICATION => "notification",
		_ => panic!("Unknown severity {}", severity),
	};

	let ty = match ty {
		gl::DEBUG_TYPE_ERROR => "error",
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behaviour",
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behaviour",
		gl::DEBUG_TYPE_PORTABILITY => "portability",
		gl::DEBUG_TYPE_PERFORMANCE => "performance",
		gl::DEBUG_TYPE_OTHER => "other",
		_ => panic!("Unknown type {}", ty),
	};

	let source = match source {
		gl::DEBUG_SOURCE_API => "api",
		gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
		gl::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
		gl::DEBUG_SOURCE_THIRD_PARTY => "third party",
		gl::DEBUG_SOURCE_APPLICATION => "application",
		gl::DEBUG_SOURCE_OTHER => "other",
		_ => panic!("Unknown source {}", source),
	};

	eprintln!("GL ERROR!");
	eprintln!("Source:   {}", source);
	eprintln!("Severity: {}", severity);
	eprintln!("Type:     {}", ty);

	unsafe {
		let msg = std::ffi::CStr::from_ptr(msg as _).to_str().unwrap();
		eprintln!("Message: {}", msg);
	}

	panic!("GL ERROR!");
}