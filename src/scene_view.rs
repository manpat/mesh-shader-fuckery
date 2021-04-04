
use std::error::Error;
use common::math::*;
use crate::{gl, mesh, perf};

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


pub struct SceneView {
	program: gl::Program,

	vertex_ssbo: gl::Buffer,
	meshlet_data_ssbo: gl::Buffer,
	num_meshlets: u32,
}

impl SceneView {
	pub fn new(gl_ctx: &gl::Context) -> Result<SceneView, Box<dyn Error>> {
		let program = gl_ctx.new_shader(&[
			(gl::raw::MESH_SHADER_NV, include_str!("shaders/scene.mesh.glsl")),
			(gl::raw::FRAGMENT_SHADER, include_str!("shaders/scene.frag.glsl")),
		]);


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

		let vertex_ssbo = gl_ctx.new_buffer();
		let meshlet_data_ssbo = gl_ctx.new_buffer();

		vertex_ssbo.upload(&mesh.vertex_data, gl::BufferUsage::Static);
		meshlet_data_ssbo.upload(&mesh.meshlet_data, gl::BufferUsage::Static);

		Ok(SceneView {
			program,
			vertex_ssbo,
			meshlet_data_ssbo,
			num_meshlets: mesh.num_meshlets as _,
		})
	}

	pub fn draw(&self, gl_ctx: &gl::Context, inst: &mut perf::Instrumenter) {
		gl_ctx.bind_shader_storage_buffer(0, self.vertex_ssbo);
		gl_ctx.bind_shader_storage_buffer(1, self.meshlet_data_ssbo);

		gl_ctx.use_program(self.program);

		inst.start_section("scene");
		gl_ctx.draw_mesh_tasks(0, self.num_meshlets);
		inst.end_section();
	}
}