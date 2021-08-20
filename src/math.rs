#[derive(Debug, Clone)]
pub struct V3
{
	pub x: f32,
	pub y: f32,
	pub z: f32
}

impl std::ops::AddAssign<&V3> for V3
{
	fn add_assign(&mut self, rhs: &V3)
	{
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl std::ops::SubAssign<&V3> for V3
{
	fn sub_assign(&mut self, rhs: &V3)
	{
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

impl std::ops::Add<&V3> for V3
{
	type Output = V3;

	fn add(self, rhs: &V3) -> Self::Output
	{
		let mut res = self.clone();
		res += rhs;
		return res;
	}
}

impl std::ops::Sub<&V3> for V3
{
	type Output = V3;

	fn sub(self, rhs: &V3) -> Self::Output
	{
		let mut res = self.clone();
		res -= rhs;
		return res;
	}
}

#[derive(Debug, Clone)]
pub struct P3(pub V3);

impl P3
{
	pub fn translate(&mut self, v3: &V3)
	{
		self.0 += v3;
	}
}

pub struct Shape
{
	pub center : P3,
	pub vertexs: Vec<P3>,
}

impl Shape
{
	pub fn create_cube(center: P3, side_length: f32) -> Shape
	{
		//NOTE(fpalacios): Displacement module from center (The ammount you move from the center to reach a vertex)
		let dmfc = side_length / 2_f32;

		let mut p1 = center.clone(); p1.translate(&V3 {x: -dmfc, y:  dmfc, z: -dmfc});
		let mut p2 = center.clone(); p2.translate(&V3 {x: -dmfc, y: -dmfc, z: -dmfc});
		let mut p3 = center.clone(); p3.translate(&V3 {x:  dmfc, y:  dmfc, z: -dmfc});
		let mut p4 = center.clone(); p4.translate(&V3 {x:  dmfc, y: -dmfc, z: -dmfc});
		let mut p5 = center.clone(); p5.translate(&V3 {x: -dmfc, y:  dmfc, z:  dmfc});
		let mut p6 = center.clone(); p6.translate(&V3 {x: -dmfc, y: -dmfc, z:  dmfc});
		let mut p7 = center.clone(); p7.translate(&V3 {x:  dmfc, y:  dmfc, z:  dmfc});
		let mut p8 = center.clone(); p8.translate(&V3 {x:  dmfc, y: -dmfc, z:  dmfc});

		return Shape
		{
			center,
			vertexs: vec![p1, p2, p3, p4, p5, p6, p7, p8]
		}
	}
}