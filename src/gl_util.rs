#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Vertex {
	pub position: [f32; 3],
	pub normal:   [f32; 3],
	pub color:    [f32; 3],
}

use na::{Isometry3, Point3, Vector3};
pub trait Camera<N> {
	fn to_isometry(&self) -> Isometry3<N>;
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Copy)]
pub struct SimpleCamera<N> {
	pub position: Point3<N>,
	pub direction: Vector3<N>,
	pub up: Vector3<N>,
}

use na::BaseFloat;
impl <N: Clone + BaseFloat> Camera<N> for SimpleCamera<N> {
	fn to_isometry(&self) -> Isometry3<N> { Isometry3::look_at_rh(&self.position, &(self.position + self.direction), &self.up) }
}

implement_vertex!(Vertex, position, normal, color);
