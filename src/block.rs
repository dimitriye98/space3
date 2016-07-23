bitflags! {
	pub flags Directions: u8 {
		const FRONT = 0b_0000_0001,
		const UP    = 0b_0000_0010,
		const RIGHT = 0b_0000_0100,
		const BACK  = 0b_0000_1000,
		const DOWN  = 0b_0001_0000,
		const LEFT  = 0b_0010_0000,
	}
}

pub trait BlockType {
	fn color(&self) -> [f32; 3];
	fn obscures(&self) -> Directions;
	fn should_render(&self) -> bool;
}

#[derive(Copy, Clone, PartialEq)]
pub struct SimpleBlock {
	pub color: [f32; 3]
}

impl BlockType for SimpleBlock {
	fn color(&self) -> [f32; 3] { self.color }
	fn obscures(&self) -> Directions { Directions::all() }
	fn should_render(&self) -> bool { true }
}

pub struct Air;

impl BlockType for Air {
	fn color(&self) -> [f32; 3] { [0.0, 0.0, 0.0] }
	fn obscures(&self) -> Directions { Directions::empty() }
	fn should_render(&self) -> bool { false }
}

pub struct Chunk<'chunk> {
	// FIXME: Encapsulate blocks field
	pub blocks: [[[&'chunk BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

const NORM_FRONT: [f32; 3] = [ 0.0,  0.0,  1.0];
const NORM_UP:    [f32; 3] = [ 0.0,  1.0,  0.0];
const NORM_RIGHT: [f32; 3] = [ 1.0,  0.0,  0.0];
const NORM_BACK:  [f32; 3] = [ 0.0,  0.0, -1.0];
const NORM_DOWN:  [f32; 3] = [ 0.0, -1.0,  0.0];
const NORM_LEFT:  [f32; 3] = [-1.0,  0.0,  0.0];

pub const CHUNK_SIZE: usize = 32;

use gl_util::Vertex;
use glium::VertexBuffer;
use glium::backend::Facade;
use glium::vertex::BufferCreationError;
impl <'chunk> Chunk<'chunk> {
	pub fn build_mesh<F: Facade>(&self, facade: &F) -> Result<VertexBuffer<Vertex>, BufferCreationError> {
		let mut data: Vec<Vertex> = Vec::new();

		for x in 0..CHUNK_SIZE {
			for y in 0..CHUNK_SIZE {
				for z in 0..CHUNK_SIZE {
					if self.blocks[x][y][z].should_render() {
						let color = self.blocks[x][y][z].color();
						if x == 0 || !self.blocks[x - 1][y][z].obscures().contains(RIGHT) {
							data.push(Vertex {
								position: [x as f32, y as f32, z as f32],
								normal:   NORM_LEFT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, y as f32, (z + 1) as f32],
								normal:   NORM_LEFT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, z as f32],
								normal:   NORM_LEFT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, z as f32],
								normal:   NORM_LEFT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, y as f32, (z + 1) as f32],
								normal:   NORM_LEFT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_LEFT,
								color:    color,
							});
						}
						if x == CHUNK_SIZE - 1 || !self.blocks[x + 1][y][z].obscures().contains(LEFT) {
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, z as f32],
								normal:   NORM_RIGHT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, (z + 1) as f32],
								normal:   NORM_RIGHT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, z as f32],
								normal:   NORM_RIGHT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_RIGHT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, (z + 1) as f32],
								normal:   NORM_RIGHT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, z as f32],
								normal:   NORM_RIGHT,
								color:    color,
							});
						}
						if y == 0 || !self.blocks[x][y - 1][z].obscures().contains(UP) {
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, z as f32],
								normal:   NORM_DOWN,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, y as f32, (z + 1) as f32],
								normal:   NORM_DOWN,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, y as f32, z as f32],
								normal:   NORM_DOWN,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, (z + 1) as f32],
								normal:   NORM_DOWN,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, y as f32, (z + 1) as f32],
								normal:   NORM_DOWN,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, z as f32],
								normal:   NORM_DOWN,
								color:    color,
							});
						}
						if y == CHUNK_SIZE - 1 || !self.blocks[x][y + 1][z].obscures().contains(DOWN) {
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, z as f32],
								normal:   NORM_UP,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_UP,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, z as f32],
								normal:   NORM_UP,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, z as f32],
								normal:   NORM_UP,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_UP,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_UP,
								color:    color,
							});
						}
						if z == 0 || !self.blocks[x][y][z - 1].obscures().contains(FRONT) {
							data.push(Vertex {
								position: [x as f32, y as f32, z as f32],
								normal:   NORM_BACK,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, z as f32],
								normal:   NORM_BACK,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, z as f32],
								normal:   NORM_BACK,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, z as f32],
								normal:   NORM_BACK,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, z as f32],
								normal:   NORM_BACK,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, z as f32],
								normal:   NORM_BACK,
								color:    color,
							});
						}
						if z == CHUNK_SIZE - 1 || !self.blocks[x][y][z + 1].obscures().contains(BACK) {
							data.push(Vertex {
								position: [x as f32, y as f32, (z + 1) as f32],
								normal:   NORM_FRONT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, (z + 1) as f32],
								normal:   NORM_FRONT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_FRONT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, y as f32, (z + 1) as f32],
								normal:   NORM_FRONT,
								color:    color,
							});
							data.push(Vertex {
								position: [(x + 1) as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_FRONT,
								color:    color,
							});
							data.push(Vertex {
								position: [x as f32, (y + 1) as f32, (z + 1) as f32],
								normal:   NORM_FRONT,
								color:    color,
							});
						}
					}
				}
			}
		}

		VertexBuffer::new(facade, &data)
	}
}

