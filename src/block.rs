use na::ToHomogeneous;

use noise::{Brownian3, Seed};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::ops::Deref;

pub struct World {
	seed: Seed,
	generator: Brownian3<f32, fn(&Seed, &[f32; 3]) -> f32>,
	chunks: RefCell<HashMap<[i64; 3], Weak<RefCell<Chunk>>>>,
}

use rand;
use noise;
use rand::Rand;
impl World {
	pub fn new() -> World {
		World {
			seed: Seed::new(12),
			generator: Brownian3::new(noise::perlin3 as fn(&Seed, &[f32; 3]) -> f32, 4).wavelength(128.0),
			chunks: RefCell::new(HashMap::new()),
		}
	}

	pub fn get_chunk(&self, x: i64, y: i64, z: i64) -> Rc<RefCell<Chunk>> {
		let opt = self.chunks.borrow().get(&[x, y, z]).and_then(Weak::upgrade);
		opt.unwrap_or_else(|| self.gen_chunk(x, y, z))
	}

	fn gen_chunk(&self, x: i64, y: i64, z: i64) -> Rc<RefCell<Chunk>> {
		let rc = Rc::new(RefCell::new(Chunk::new([[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE])));

		{
			let mut chunk = rc.borrow_mut();

			for index_x in 0..CHUNK_SIZE {
				for index_y in 0..CHUNK_SIZE {
					for index_z in 0..CHUNK_SIZE {
						let (block_x, block_y, block_z) = (CHUNK_SIZE as i64 * x + index_x as i64, CHUNK_SIZE as i64 * y + index_y as i64, CHUNK_SIZE as i64 * z + index_z as i64);

						let mut density = -block_z as f32 / 128.0;
						density += self.generator.apply(&self.seed, &[block_x as f32, block_y as f32, block_z as f32]);

						if density > 0.0 {
							chunk.blocks[index_x][index_y][index_z] = 1;
						}
					}
				}
			}

			self.chunks.borrow_mut().insert([x, y, z], Rc::downgrade(&rc));
		}
		rc
	}
}

use glium::Display;
pub trait Region {
	fn draw(display: &Display);
}

use ndarray::{Array, Ix};
pub struct CuboidRegion {
	start_pos: [i64; 3],
	chunks: Array<Rc<RefCell<Chunk>>, (Ix, Ix, Ix)>,
}

use engine::DrawService;
use ndarray::Axis;
use na::Isometry3;
impl CuboidRegion {
	pub fn new(
		world: &World,
		start_x: i64, start_y: i64, start_z: i64,
		end_x: i64, end_y: i64, end_z: i64
	) -> CuboidRegion {
		let (s_x, e_x) = if end_x >= start_x { (start_x, end_x + 1) } else { (end_x, start_x + 1) };
		let (s_y, e_y) = if end_y >= start_y { (start_y, end_y + 1) } else { (end_y, start_y + 1) };
		let (s_z, e_z) = if end_z >= start_z { (start_z, end_z + 1) } else { (end_z, start_z + 1) };

		let mut region = Vec::with_capacity((e_x - s_x) as usize * (e_y - s_y) as usize * (e_z - s_z) as usize);
		for x in s_x..e_x {
			for y in s_y..e_y {
				for z in s_z..e_z {
					region.push(world.get_chunk(x, y, z));
				}
			}
		}

		CuboidRegion {
			start_pos: [s_x, s_y, s_z],
			chunks: Array::from_shape_vec(((e_x - s_x) as usize, (e_y - s_y) as usize, (e_z - s_z) as usize), region).unwrap(),
		}
	}

	pub fn draw(&self, block_render_data: &[BlockRenderData], draw_service: &mut DrawService, view: Matrix4<f32>) {
		let mut x = self.start_pos[0];
		for slice_x in self.chunks.axis_iter(Axis(0)) {
			let mut y = self.start_pos[1];
			for slice_y in slice_x.axis_iter(Axis(0)) {
				let mut z = self.start_pos[2];
				for chunk in slice_y.iter() {
					let (vertices, indices) = chunk.borrow().build_mesh(block_render_data, [Option::None; 6], draw_service.facade()).unwrap();

					draw_service.draw_buffer(
						&(view * Matrix4::new(1.0, 0.0, 0.0, (x * CHUNK_SIZE as i64) as f32,
						                      0.0, 1.0, 0.0, (y * CHUNK_SIZE as i64) as f32,
						                      0.0, 0.0, 1.0, (z * CHUNK_SIZE as i64) as f32,
						                      0.0, 0.0, 0.0, 1.0)),
						&*vertices,
						&*indices
					);

					z += 1;
				}
				y += 1;
			}
			x += 1;
		}
	}
}

// FIXME: Encapsulation
pub struct Chunk {
	pub blocks: [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
	mesh: RefCell<Option<(Rc<VertexBuffer<Vertex>>, Rc<IndexBuffer<u16>>)>>
}

#[derive(Debug, Copy, Clone)]
pub enum NormalDirection {
	Up,
	Down,
	Left,
	Right,
	Front,
	Back,
}

use na::Vector3;
impl NormalDirection {
	#[inline]
	fn to_vec_arr(&self) -> [f32; 3] {
		use block::NormalDirection as ND;
		match self {
			&ND::Front => [ 0.0,  1.0,  0.0],
			&ND::Up    => [ 0.0,  0.0,  1.0],
			&ND::Right => [ 1.0,  0.0,  0.0],
			&ND::Back  => [ 0.0, -1.0,  0.0],
			&ND::Down  => [ 0.0,  0.0, -1.0],
			&ND::Left  => [-1.0,  0.0,  0.0],
		}
	}

	#[inline]
	fn to_vec3(&self) -> Vector3<f32> {
		use block::NormalDirection as ND;
		match self {
			&ND::Front => Vector3::new( 0.0,  1.0,  0.0),
			&ND::Up    => Vector3::new( 0.0,  0.0,  1.0),
			&ND::Right => Vector3::new( 1.0,  0.0,  0.0),
			&ND::Back  => Vector3::new( 0.0, -1.0,  0.0),
			&ND::Down  => Vector3::new( 0.0,  0.0, -1.0),
			&ND::Left  => Vector3::new(-1.0,  0.0,  0.0),
		}
	}

	#[inline]
	fn to_index(&self) -> usize {
		use block::NormalDirection as ND;
		match self {
			&ND::Front => 0,
			&ND::Up    => 1,
			&ND::Right => 2,
			&ND::Back  => 3,
			&ND::Down  => 4,
			&ND::Left  => 5,
		}
	}
}

use std::ops::Neg;
impl Neg for NormalDirection {
	type Output = NormalDirection;

	#[inline]
	fn neg(self) -> NormalDirection {
		use block::NormalDirection as ND;
		match self {
			ND::Front => ND::Back,
			ND::Back  => ND::Front,

			ND::Left  => ND::Right,
			ND::Right => ND::Left,

			ND::Up    => ND::Down,
			ND::Down  => ND::Up,
		}
	}
}

impl <'a> Neg for &'a NormalDirection {
	type Output = NormalDirection;

	#[inline]
	fn neg(self) -> NormalDirection {
		use block::NormalDirection as ND;
		match self {
			&ND::Front => ND::Back,
			&ND::Back  => ND::Front,

			&ND::Left  => ND::Right,
			&ND::Right => ND::Left,

			&ND::Up    => ND::Down,
			&ND::Down  => ND::Up,
		}
	}
}

pub struct BlockRenderData {
	pub obscures: u8,
	pub color: [f32; 3],
	pub should_render: bool,
}

impl BlockRenderData {
	fn obscures(&self, dir: &NormalDirection) -> bool {
		use block::NormalDirection as ND;
		let bit = match dir {
			&ND::Front => 0b000001,
			&ND::Up    => 0b000010,
			&ND::Right => 0b000100,
			&ND::Back  => 0b001000,
			&ND::Down  => 0b010000,
			&ND::Left  => 0b100000,
		};
		self.obscures & bit != 0
	}
}

use glium::vertex::BufferCreationError as VertexBufferCreationError;
use glium::index::BufferCreationError as IndexBufferCreationError;

#[derive(Debug, Copy, Clone)]
pub enum MeshCreationError {
	VertexBufferCreationFailed(VertexBufferCreationError),
	IndexBufferCreationFailed(IndexBufferCreationError),
}

impl From<VertexBufferCreationError> for MeshCreationError {
	fn from(err: VertexBufferCreationError) -> MeshCreationError {
		MeshCreationError::VertexBufferCreationFailed(err)
	}
}

impl From<IndexBufferCreationError> for MeshCreationError {
	fn from(err: IndexBufferCreationError) -> MeshCreationError {
		MeshCreationError::IndexBufferCreationFailed(err)
	}
}

pub const CHUNK_SIZE: usize = 32;

use glium::{VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::backend::Facade;

use na::{Matrix3, Matrix4};

use gl_util::Vertex;
impl Chunk {
	pub fn new(blocks: [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]) -> Chunk {
		Chunk {
			blocks: blocks,
			mesh: RefCell::new(Option::None),
		}
	}

	pub fn build_mesh<F: Facade>(&self, block_render_data: &[BlockRenderData], adj_chunks: [Option<&Chunk>; 6], facade: &F)
			-> Result<(Rc<VertexBuffer<Vertex>>, Rc<IndexBuffer<u16>>), MeshCreationError> {
		use block::NormalDirection as ND;

		if let &Some((ref v, ref i)) = &*self.mesh.borrow() {
			return Ok((v.clone(), i.clone()));
		}

		let mut data: Vec<Vertex> = Vec::new();
		let mut indices: Vec<u16> = Vec::new();

		let mut quad_start = 0;
		for up_dir in [ND::Up, ND::Down, ND::Left, ND::Right, ND::Front, ND::Back].into_iter() {
			let up_vec3 = up_dir.to_vec_arr();
			for w in 0..CHUNK_SIZE {
				let mut slice: [[Option<[f32; 3]>; CHUNK_SIZE]; CHUNK_SIZE] = [[None; CHUNK_SIZE]; CHUNK_SIZE];

				for u in 0..CHUNK_SIZE {
					for v in 0..CHUNK_SIZE {
						let (x, y, z) = match up_dir {
							&ND::Up    => (&u, &v, &w),
							&ND::Down  => (&v, &u, &w),

							&ND::Left  => (&w, &v, &u),
							&ND::Right => (&w, &u, &v),

							&ND::Front => (&v, &w, &u),
							&ND::Back  => (&u, &w, &v),
						};

						let (x_offset, y_offset, z_offset) = match up_dir {
							&ND::Up    => (0, 0, 1),
							&ND::Down  => (0, 0, -1isize as usize),

							&ND::Left  => (-1isize as usize, 0, 0),
							&ND::Right => (1, 0, 0),

							&ND::Front => (0, 1, 0),
							&ND::Back  => (0, -1isize as usize, 0),
						};

						if !block_render_data[self.blocks[*x][*y][*z]].should_render {
							slice[u][v] = None;
							continue;
						}

						let (query_x, query_y, query_z) = (x.wrapping_add(x_offset), y.wrapping_add(y_offset), z.wrapping_add(z_offset));

						slice[u][v] = if query_x >= CHUNK_SIZE || query_y >= CHUNK_SIZE || query_z >= CHUNK_SIZE {
							if let Some(chunk) = adj_chunks[(-up_dir).to_index()] {
								if !block_render_data[chunk.blocks[query_x % CHUNK_SIZE][query_y % CHUNK_SIZE][query_z % CHUNK_SIZE]].obscures(&-up_dir) {
									Some(block_render_data[self.blocks[*x][*y][*z]].color)
								} else {
									None
								}
							} else {
								Some(block_render_data[self.blocks[*x][*y][*z]].color)
							}
						} else {
							if !block_render_data[self.blocks[query_x][query_y][query_z]].obscures(&-up_dir) {
								Some(block_render_data[self.blocks[*x][*y][*z]].color)
							} else {
								None
							}
						};
					}
				}

				let (mut u, mut v) = (0, 0);
				while v < CHUNK_SIZE {
					while u < CHUNK_SIZE {
						match slice[u][v] {
							None => { u += 1; },
							Some(color) => {
								let mut width: usize = 1;
								while u + width < CHUNK_SIZE && slice[u + width][v] == Some(color) {
									width += 1;
								}

								let mut height: usize = CHUNK_SIZE - v;
								'outer: for h in 1..(CHUNK_SIZE - v) {
									for k in 0..width {
										if slice[u + k][v + h] != Some(color) {
											height = h;
											break 'outer;
										}
									}
								}

								for j in 0..height {
									for i in 0..width {
										slice[u + i][v + j] = None;
									}
								}

								let w_offset = match up_dir {
									&ND::Up    => 1,
									&ND::Down  => 0,

									&ND::Left  => 0,
									&ND::Right => 1,

									&ND::Front => 1,
									&ND::Back  => 0,
								};

								let (u_float, v_float, w_float, u_width_float, v_height_float) = (u as f32, v as f32, (w + w_offset) as f32, (u + width) as f32, (v + height) as f32);

								data.push(Vertex {
									position: match up_dir {
										&ND::Up    => [u_float, v_height_float, w_float],
										&ND::Down  => [v_height_float, u_float, w_float],

										&ND::Left  => [w_float, v_height_float, u_float],
										&ND::Right => [w_float, u_float, v_height_float],

										&ND::Front => [v_height_float, w_float, u_float],
										&ND::Back  => [u_float, w_float, v_height_float],
									},
									normal: up_vec3,
									color: color,
								});

								data.push(Vertex {
									position: match up_dir {
										&ND::Up    => [u_float, v_float, w_float],
										&ND::Down  => [v_float, u_float, w_float],

										&ND::Left  => [w_float, v_float, u_float],
										&ND::Right => [w_float, u_float, v_float],

										&ND::Front => [v_float, w_float, u_float],
										&ND::Back  => [u_float, w_float, v_float],
									},
									normal: up_vec3,
									color: color,
								});

								data.push(Vertex {
									position: match up_dir {
										&ND::Up    => [u_width_float, v_height_float, w_float],
										&ND::Down  => [v_height_float, u_width_float, w_float],

										&ND::Left  => [w_float, v_height_float, u_width_float],
										&ND::Right => [w_float, u_width_float, v_height_float],

										&ND::Front => [v_height_float, w_float, u_width_float],
										&ND::Back  => [u_width_float, w_float, v_height_float],
									},
									normal: up_vec3,
									color: color,
								});

								data.push(Vertex {
									position: match up_dir {
										&ND::Up    => [u_width_float, v_float, w_float],
										&ND::Down  => [v_float, u_width_float, w_float],

										&ND::Left  => [w_float, v_float, u_width_float],
										&ND::Right => [w_float, u_width_float, v_float],

										&ND::Front => [v_float, w_float, u_width_float],
										&ND::Back  => [u_width_float, w_float, v_float],
									},
									normal: up_vec3,
									color: color,
								});

								indices.push(quad_start + 0);
								indices.push(quad_start + 1);
								indices.push(quad_start + 2);
								indices.push(quad_start + 3);
								indices.push(quad_start + 2);
								indices.push(quad_start + 1);

								quad_start += 4;
								u += width;
							}
						}
					}

					v += 1;
					u = 0;
				}
			}
		}

		let res = VertexBuffer::new(facade, &data)
			.map(|v| Rc::new(v))
			.map_err(|e| MeshCreationError::from(e))
			.and_then(|v| IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)
				.map(|i| (v, Rc::new(i)))
				.map_err(|e| MeshCreationError::from(e))
			);

		if let Ok(ref mesh) = res {
			let mut cache = self.mesh.borrow_mut();
			*cache = Some(mesh.clone());
		};

		res

	}
}

