use crate::gl;


#[derive(Copy, Clone, Debug)]
enum State {
	Recording,
	Waiting,
}

pub struct Instrumenter {
	section_cache: Vec<Section>,
	recording_section: Option<Section>,
	waiting_sections: Vec<Section>,
	state: State,
}


impl Instrumenter {
	pub fn new(gl_ctx: &gl::Context) -> Instrumenter {
		let mut section_cache = Vec::new();

		for _ in 0..20 {
			section_cache.push(Section::new(gl_ctx));
		}

		Instrumenter {
			section_cache,
			recording_section: None,
			waiting_sections: Vec::new(),
			state: State::Recording,
		}
	}


	pub fn start_section(&mut self, name: &str) {
		match self.state {
			State::Recording => {},
			State::Waiting => return,
		};

		if self.recording_section.is_some() {
			self.end_section();
		}

		let mut section = self.section_cache.pop()
			.expect("Query cache empty!");

		section.start(name.into());

		self.recording_section = Some(section);
	}

	pub fn end_section(&mut self) {
		match self.state {
			State::Recording => {},
			State::Waiting => return,
		};

		let section = self.recording_section.take()
			.expect("Mismatched start/end query section!");

		Section::end();

		self.waiting_sections.push(section);
	}


	pub fn end_frame(&mut self) {
		if self.recording_section.is_some() {
			self.end_section();
		}

		self.state = State::Waiting;

		let queries_ready = self.waiting_sections.iter()
			.all(&Section::ready);

		if queries_ready {
			let mut total_time = 0.0f64;
			let mut total_tris = 0usize;

			for section in self.waiting_sections.drain(..) {
				let (time_nanos, triangles) = section.result();
				let time_ms = time_nanos as f64 / 1000_000.0;
				print!("[{}: {}tris {:.3}ms] ", section.name, triangles, time_ms);

				total_time += time_ms;
				total_tris += triangles;

				self.section_cache.push(section);
			}

			println!("[[total: {}tris {:.3}ms]]", total_tris, total_time);

			self.state = State::Recording;
		}
	}
}




struct Section {
	name: String,
	timer_handle: u32,
	geo_handle: u32,
}

impl Section {
	fn new(_gl_ctx: &gl::Context) -> Section {
		unsafe {
			let mut handles = [0; 2];
			gl::raw::GenQueries(2, handles.as_mut_ptr());

			let [timer_handle, geo_handle] = handles;
			Section {
				name: String::new(),
				timer_handle,
				geo_handle,
			}
		}
	}

	fn start(&mut self, name: String) {
		unsafe {
			self.name = name;
			gl::raw::BeginQuery(gl::raw::PRIMITIVES_GENERATED, self.geo_handle);
			gl::raw::BeginQuery(gl::raw::TIME_ELAPSED, self.timer_handle);
		}
	}

	fn end() {
		unsafe {
			gl::raw::EndQuery(gl::raw::PRIMITIVES_GENERATED);
			gl::raw::EndQuery(gl::raw::TIME_ELAPSED);
		}
	}

	fn ready(&self) -> bool {
		unsafe {
			let mut timer_ready = 0;
			let mut geo_ready = 0;
			gl::raw::GetQueryObjectiv(self.timer_handle, gl::raw::QUERY_RESULT_AVAILABLE, &mut timer_ready);
			gl::raw::GetQueryObjectiv(self.geo_handle, gl::raw::QUERY_RESULT_AVAILABLE, &mut geo_ready);
			(timer_ready != 0) && (geo_ready != 0)
		}
	}

	fn result(&self) -> (usize, usize) {
		unsafe {
			let mut timer_value = 0;
			let mut geo_value = 0;
			gl::raw::GetQueryObjectiv(self.timer_handle, gl::raw::QUERY_RESULT, &mut timer_value);
			gl::raw::GetQueryObjectiv(self.geo_handle, gl::raw::QUERY_RESULT, &mut geo_value);
			(timer_value as usize, geo_value as usize)
		}
	}
}
