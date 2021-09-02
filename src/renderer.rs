glium::implement_vertex!(Vertex2D, position, texture_coords);
#[derive(Copy, Clone)]
pub struct Vertex2D
{
	pub position      : [f32; 2],
	pub texture_coords: [f32; 2],
}

glium::implement_vertex!(Vertex3D, position, color);
#[derive(Copy, Clone)]
pub struct Vertex3D
{
	pub position: [f32; 3],
	pub color   : [f32; 3],
}

pub struct Model2D
{
	pub vertex_buffer         : glium::VertexBuffer<Vertex2D>,
	pub index_buffer          : glium::IndexBuffer<u16>,
	pub texture               : std::rc::Rc<glium::texture::CompressedSrgbTexture2d>,
	pub texture_shader_program: glium::Program,
}

pub struct Model3D
{
	pub rotation_matrix    : nalgebra::Matrix4<f32>,
	pub translation_vector : nalgebra::Vector3<f32>,
	pub vertex_buffer      : glium::VertexBuffer<Vertex3D>,
	pub index_buffer       : glium::IndexBuffer<u16>,
	pub fill_shader_program: glium::Program,
	pub wire_shader_program: glium::Program,
}

pub struct Animation2DStep
{
	pub texture  : std::rc::Rc<glium::texture::CompressedSrgbTexture2d>,
	pub duration : std::time::Duration,
	pub next_step: Option<std::rc::Rc<Animation2DStep>>,
}

pub struct Animation2D
{
	pub model           : Model2D,
	pub timer           : std::time::Duration,
	pub root_steps_chain: std::rc::Rc<Animation2DStep>,
	pub current_step    : std::rc::Rc<Animation2DStep>,
}

impl Animation2D
{
	pub fn update(&mut self, delta: &std::time::Duration)
	{
		self.timer += *delta;

		while self.timer >= self.current_step.duration
		{
			self.timer -= self.current_step.duration;
			if let Some(next_step) = &self.current_step.next_step
			{
				self.current_step = next_step.clone();
			}
			else
			{
				self.current_step = self.root_steps_chain.clone();
			}

			self.model.texture = self.current_step.texture.clone();
		}
	}
}

pub fn draw_animation_2d(display: &glium::Display, animation: &Animation2D)
{
	draw_model_2d(display, &animation.model);
}

pub fn draw_model_2d(display: &glium::Display, model: &Model2D)
{
	let uniforms = glium::uniform!
	{
		texture2d: &*model.texture
	};

	let mut target = display.draw();

	glium::Surface::clear_color(&mut target, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
	glium::Surface::clear_depth_and_stencil(&mut target, 0.0_f32, 0_i32);
	glium::Surface::clear_all_srgb(&mut target, (0.0, 0.0, 0.0, 0.0), 0.0, 0);

	let mut draw_parameters = glium::DrawParameters::default();
	draw_parameters.blend.alpha = glium::BlendingFunction::AlwaysReplace;
	draw_parameters.blend.color = glium::BlendingFunction::AlwaysReplace;

	glium::Surface::draw(
		&mut target,
		&model.vertex_buffer,
		&model.index_buffer,
		&model.texture_shader_program,
		&uniforms,
		&draw_parameters
	).unwrap();

	target.finish().unwrap();
}

pub fn draw_model_3d(display: &glium::Display, model: &Model3D)
{
	let uniforms = glium::uniform!
	{
		rotation:
		[
			[model.rotation_matrix[ 0], model.rotation_matrix[ 1], model.rotation_matrix[ 2], model.rotation_matrix[ 3]],
			[model.rotation_matrix[ 4], model.rotation_matrix[ 5], model.rotation_matrix[ 6], model.rotation_matrix[ 7]],
			[model.rotation_matrix[ 8], model.rotation_matrix[ 9], model.rotation_matrix[10], model.rotation_matrix[11]],
			[model.rotation_matrix[12], model.rotation_matrix[13], model.rotation_matrix[14], model.rotation_matrix[15]],
		],
		translation: [model.translation_vector.x, model.translation_vector.y, model.translation_vector.z, 0_f32]
	};

	let mut target = display.draw();
	glium::Surface::clear_color(&mut target, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);

	let mut draw_parameters = glium::DrawParameters::default();

	draw_parameters.polygon_mode = glium::PolygonMode::Fill;

	glium::Surface::draw(
		&mut target,
		&model.vertex_buffer,
		&model.index_buffer,
		&model.fill_shader_program,
		&uniforms,
		&draw_parameters
	).unwrap();

	draw_parameters.polygon_mode = glium::PolygonMode::Line;

	glium::Surface::draw(
		&mut target,
		&model.vertex_buffer,
		&model.index_buffer,
		&model.wire_shader_program,
		&uniforms,
		&draw_parameters
	).unwrap();

	target.finish().unwrap();
}