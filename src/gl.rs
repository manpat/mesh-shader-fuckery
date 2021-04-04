use std::collections::HashMap;


pub mod raw {
	include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}


pub struct Context {
	_sdl_ctx: sdl2::video::GLContext,
	imports: HashMap<String, String>,
}

#[derive(Copy, Clone, Debug)]
pub struct Program (u32);


#[derive(Copy, Clone, Debug)]
pub enum BufferUsage {
	Static,
	Dynamic,
	Stream,
}

#[derive(Copy, Clone, Debug)]
pub struct Buffer (u32);

#[derive(Copy, Clone, Debug)]
pub struct Texture (u32);



impl Context {
	pub fn new(sdl_ctx: sdl2::video::GLContext) -> Self {
		unsafe {
			raw::DebugMessageCallback(Some(gl_message_callback), std::ptr::null());
			raw::Enable(raw::DEBUG_OUTPUT_SYNCHRONOUS);
			raw::Enable(raw::PROGRAM_POINT_SIZE);

			raw::Enable(raw::DEPTH_TEST);
			raw::Enable(raw::BLEND);
			raw::BlendFunc(raw::DST_COLOR, raw::ZERO);
			raw::BlendEquation(raw::FUNC_ADD);

			// Disable performance messages
			raw::DebugMessageControl(
				raw::DONT_CARE,
				raw::DEBUG_TYPE_PERFORMANCE,
				raw::DONT_CARE,
				0, std::ptr::null(),
				0 // false
			);

			// Disable notification messages
			raw::DebugMessageControl(
				raw::DONT_CARE,
				raw::DONT_CARE,
				raw::DEBUG_SEVERITY_NOTIFICATION,
				0, std::ptr::null(),
				0 // false
			);
		}

		Context {
			_sdl_ctx: sdl_ctx,
			imports: HashMap::new(),
		}
	}

	pub fn new_buffer(&self) -> Buffer {
		unsafe {
			let mut buf = 0;
			raw::CreateBuffers(1, &mut buf);
			Buffer(buf)
		}
	}

	pub fn new_texture(&self, width: u32, height: u32, format: u32) -> Texture {
		unsafe {
			let mut tex = 0;
			raw::CreateTextures(raw::TEXTURE_2D, 1, &mut tex);
			raw::TextureStorage2D(tex, 1, format, 4096, 4096);
			raw::TextureParameteri(tex, raw::TEXTURE_MIN_FILTER, raw::LINEAR as _);
			Texture(tex)
		}
	}

	pub fn bind_uniform_buffer(&self, binding: u32, buffer: Buffer) {
		unsafe {
			raw::BindBufferBase(raw::UNIFORM_BUFFER, binding, buffer.0);
		}
	}

	pub fn bind_shader_storage_buffer(&self, binding: u32, buffer: Buffer) {
		unsafe {
			raw::BindBufferBase(raw::SHADER_STORAGE_BUFFER, binding, buffer.0);
		}
	}

	pub fn bind_image_rw(&self, binding: u32, texture: Texture, format: u32) {
		unsafe {
			let (level, layered, layer) = (0, 0, 0);
			raw::BindImageTexture(binding, texture.0, level, layered, layer, raw::READ_WRITE, format);
		}
	}

	pub fn bind_texture(&self, binding: u32, texture: Texture) {
		unsafe {
			raw::BindTextureUnit(binding, texture.0);
		}
	}


	pub fn add_shader_import(&mut self, name: impl Into<String>, src: impl Into<String>) {
		let existing_import_src = self.imports.insert(name.into(), src.into());
		assert!(existing_import_src.is_none());
	}

	fn resolve_imports(&self, mut src: &str) -> String {
		let search_pattern = "#import";
		let mut result = String::with_capacity(src.len());

		while !src.is_empty() {
			let (prefix, suffix) = match src.split_once(search_pattern) {
				Some(pair) => pair,
				None => {
					result.push_str(src);
					break
				}
			};

			let (import_name, suffix) = suffix.split_once('\n')
				.expect("Expected '#common <name>'");
			src = suffix;

			let import_name = import_name.trim();
			let import_str = self.imports.get(import_name)
				.expect("Unknown import");

			result.push_str(prefix);
			result.push_str(import_str);
		}

		result
	}

	pub fn new_shader(&self, shaders: &[(u32, &str)]) -> Program {
		use std::ffi::CString;
		use std::str;

		unsafe {
			let program_handle = raw::CreateProgram();

			for &(ty, src) in shaders {
				let src = self.resolve_imports(&src);
				let src = CString::new(src.as_bytes()).unwrap();

				let shader_handle = raw::CreateShader(ty);

				raw::ShaderSource(shader_handle, 1, &src.as_ptr(), std::ptr::null());
				raw::CompileShader(shader_handle);

				let mut status = 0;
				raw::GetShaderiv(shader_handle, raw::COMPILE_STATUS, &mut status);

				if status == 0 {
					let mut length = 0;
					raw::GetShaderiv(shader_handle, raw::INFO_LOG_LENGTH, &mut length);

					let mut buffer = vec![0u8; length as usize];
					raw::GetShaderInfoLog(
						shader_handle,
						length,
						std::ptr::null_mut(),
						buffer.as_mut_ptr() as *mut _
					);

					let error = str::from_utf8(&buffer[..buffer.len()-1]).unwrap();

					panic!("Shader compile failed!\n{}", error);
				}

				raw::AttachShader(program_handle, shader_handle);
				raw::DeleteShader(shader_handle);
			}

			raw::LinkProgram(program_handle);

			let mut status = 0;
			raw::GetProgramiv(program_handle, raw::LINK_STATUS, &mut status);

			if status == 0 {
				let mut buf = [0u8; 1024];
				let mut len = 0;
				raw::GetProgramInfoLog(program_handle, buf.len() as _, &mut len, buf.as_mut_ptr() as _);

				panic!("shader link failed: {}", std::str::from_utf8(&buf[..len as usize]).unwrap());
			}

			Program(program_handle)
		}
	}

	pub fn use_program(&self, program: Program) {
		unsafe {
			raw::UseProgram(program.0);
		}
	}

	pub fn draw_mesh_tasks(&self, offset: u32, count: u32) {
		unsafe {
			raw::DrawMeshTasksNV(offset, count);
		}
	}

	pub fn dispatch_compute(&self, x: u32, y: u32, z: u32) {
		unsafe {
			raw::DispatchCompute(x, y, z);
		}
	}
}


impl Buffer {
	pub fn upload<T>(&self, data: &[T], usage: BufferUsage) {
		let usage = match usage {
			BufferUsage::Static => raw::STATIC_DRAW,
			BufferUsage::Dynamic => raw::DYNAMIC_DRAW,
			BufferUsage::Stream => raw::STREAM_DRAW,
		};

		let size_bytes = data.len() * std::mem::size_of::<T>();

		unsafe {
			raw::NamedBufferData(
				self.0,
				size_bytes as _,
				data.as_ptr() as *const _,
				usage
			);
		}
	}
}


impl Texture {
	pub fn clear(&self) {
		unsafe {
			raw::ClearTexImage(self.0, 0, raw::RED, raw::FLOAT, &0.0f32 as *const f32 as _);
		}
	}
}





extern "system" fn gl_message_callback(source: u32, ty: u32, _id: u32, severity: u32,
	_length: i32, msg: *const i8, _ud: *mut std::ffi::c_void)
{
	let severity = match severity {
		raw::DEBUG_SEVERITY_LOW => "low",
		raw::DEBUG_SEVERITY_MEDIUM => "medium",
		raw::DEBUG_SEVERITY_HIGH => "high",
		raw::DEBUG_SEVERITY_NOTIFICATION => "notification",
		_ => panic!("Unknown severity {}", severity),
	};

	let ty = match ty {
		raw::DEBUG_TYPE_ERROR => "error",
		raw::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behaviour",
		raw::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behaviour",
		raw::DEBUG_TYPE_PORTABILITY => "portability",
		raw::DEBUG_TYPE_PERFORMANCE => "performance",
		raw::DEBUG_TYPE_OTHER => "other",
		_ => panic!("Unknown type {}", ty),
	};

	let source = match source {
		raw::DEBUG_SOURCE_API => "api",
		raw::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
		raw::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
		raw::DEBUG_SOURCE_THIRD_PARTY => "third party",
		raw::DEBUG_SOURCE_APPLICATION => "application",
		raw::DEBUG_SOURCE_OTHER => "other",
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