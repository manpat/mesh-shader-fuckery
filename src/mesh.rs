

#[derive(Debug)]
pub struct MeshData<V> {
	pub vertex_data: Vec<V>,
	pub meshlet_data: Vec<u8>,
	pub num_meshlets: usize,
}




const MAX_MESHLET_TRIANGLES: usize = 126;
const MAX_MESHLET_VERTICES: usize = 64;


pub struct MeshletBuilder<V> {
	vertices: Vec<V>,

	meshlet_descriptors: Vec<MeshletDescriptor>,
	vertex_indices: Vec<u32>,
	primitive_indices: Vec<u8>,

	vertex_begin: usize,
	primitive_begin: usize,
}

impl<V: Clone> MeshletBuilder<V> {
	pub fn new() -> Self {
		MeshletBuilder {
			vertices: Vec::new(),

			meshlet_descriptors: Vec::new(),
			vertex_indices: Vec::new(),
			primitive_indices: Vec::new(),

			vertex_begin: 0,
			primitive_begin: 0,
		}
	}

	pub fn append(&mut self, vertices: &[V], triangle_indices: &[u16]) {
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


	pub fn build(mut self) -> MeshData<V> {
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


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MeshletDataHeader {
	vertex_indices_offset: u32,
	primitive_indices_offset: u32,
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
