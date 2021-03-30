
pub mod raw {
	include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}


pub struct GlContext {
	_sdl_ctx: sdl2::video::GLContext,
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



impl GlContext {
	pub fn new(sdl_ctx: sdl2::video::GLContext) -> Self {
		unsafe {
			raw::DebugMessageCallback(Some(gl_message_callback), std::ptr::null());
			raw::Enable(raw::DEBUG_OUTPUT_SYNCHRONOUS);
			raw::Enable(raw::DEPTH_TEST);

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

		GlContext {
			_sdl_ctx: sdl_ctx
		}
	}

	pub fn new_buffer(&self) -> Buffer {
		unsafe {
			let mut buf = 0;
			raw::CreateBuffers(1, &mut buf);
			Buffer(buf)
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

	pub fn new_shader(&self, shaders: &[(u32, &str)]) -> Program {
		use std::ffi::CString;
		use std::str;

		unsafe {
			let program_handle = raw::CreateProgram();

			for &(ty, src) in shaders {
				let shader_handle = raw::CreateShader(ty);
				let src = CString::new(src.as_bytes()).unwrap();

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