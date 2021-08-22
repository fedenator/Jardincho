use glium::program;

const VEL_TRANSLATION_MODULE: f32 = 0.0001_f32;
const VEL_ROTATION_MODULE   : f32 = 0.0001_f32;

pub struct Cube
{
	object     : crate::renderer::Object,
	velocity   : nalgebra::Vector3<f32>,
	orientation: (f32, f32, f32),
}

impl Cube
{
	pub fn new(display: &glium::Display) -> Cube
	{
		let vertex_buffer = glium::VertexBuffer::new(
			display,
			&[
				crate::renderer::Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
				crate::renderer::Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
				crate::renderer::Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
				crate::renderer::Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
				crate::renderer::Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
				crate::renderer::Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
				crate::renderer::Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
				crate::renderer::Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
				// crate::renderer::Vertex { position: [-0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
				// crate::renderer::Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 0.0, 1.0] },
				// crate::renderer::Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
				// crate::renderer::Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
				// crate::renderer::Vertex { position: [-0.5, -0.5, -0.5], color: [0.5, 0.5, 1.0] },
				// crate::renderer::Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.5, 0.5] },
				// crate::renderer::Vertex { position: [ 0.5,  0.5, -0.5], color: [0.5, 0.5, 1.0] },
				// crate::renderer::Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
			]
		).unwrap();

		let index_buffer = glium::IndexBuffer::new(
			display,
			glium::index::PrimitiveType::TrianglesList,
			&[
				// Frente
				0_u16, 1_u16, 2_u16,
				2_u16, 3_u16, 0_u16,

				// Derecha
				1_u16, 5_u16, 6_u16,
				6_u16, 2_u16, 1_u16,

				// Izquierda
				4_u16, 0_u16, 3_u16,
				3_u16, 7_u16, 4_u16,

				// Atras
				5_u16, 4_u16, 7_u16,
				7_u16, 6_u16, 5_u16,

				// Arriba
				3_u16, 2_u16, 6_u16,
				6_u16, 7_u16, 3_u16,

				// Abajo
				4_u16, 5_u16, 1_u16,
				1_u16, 0_u16, 4_u16,
			],
		).unwrap();

		let fill_shader_program = glium::program!(
			display,
			140 =>
			{
				vertex  : include_str!("../shaders/460/FillVertShader.glsl"),
				fragment: include_str!("../shaders/460/FragShader.glsl")
			},
		).unwrap();

		let wire_shader_program = glium::program!(
			display,
			140 =>
			{
				vertex  : include_str!("../shaders/460/WireVertShader.glsl"),
				fragment: include_str!("../shaders/460/FragShader.glsl")
			},
		).unwrap();


		return Cube
		{
			velocity: nalgebra::Vector3::new(VEL_TRANSLATION_MODULE, 0_f32, 0_f32),
			orientation: (0_f32, 0_f32, 0_f32),
			object  : crate::renderer::Object
			{
				translation_vector : nalgebra::Vector3::zeros(),
				rotation_matrix    : nalgebra::Matrix4::from_euler_angles(0.8_f32, 0.8_f32, 0.8_f32),
				vertex_buffer      : vertex_buffer,
				index_buffer       : index_buffer,
				fill_shader_program: fill_shader_program,
				wire_shader_program: wire_shader_program,
			}
		};
	}

	pub fn update(&mut self)
	{
		self.object.translation_vector += &self.velocity;
		if      self.object.translation_vector.x + 0.5_f32 >=  1_f32 { self.velocity.x = -VEL_TRANSLATION_MODULE; }
		else if self.object.translation_vector.x - 0.5_f32 <= -1_f32 { self.velocity.x =  VEL_TRANSLATION_MODULE; }

		let (x, y, z) = &mut self.orientation;

		*x += VEL_ROTATION_MODULE;
		*y += VEL_ROTATION_MODULE;
		*z += VEL_ROTATION_MODULE;

		self.object.rotation_matrix = nalgebra::Matrix4::from_euler_angles(*x, *y, *z);
	}

	pub fn draw(&self, display: &glium::Display)
	{
		crate::renderer::draw(&display, &self.object);
	}
}

pub struct World
{
	pub cube: Cube,
}

impl World
{
	pub fn new(display: &glium::Display) -> World
	{
		return World
		{
			cube: Cube::new(display)
		};
	}

	pub fn update(&mut self)
	{
		self.cube.update();
	}

	pub fn draw(&self, display: &glium::Display)
	{
		self.cube.draw(display);
	}
}