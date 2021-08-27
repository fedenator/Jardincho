use glium::program;
pub struct Plant
{
	object_2d: crate::renderer::Object2D,
}

impl Plant
{
	fn update(&mut self)
	{

	}

	pub fn draw(&self, display: &glium::Display)
	{
		crate::renderer::draw_object_2d(&display, &self.object_2d);
	}
}

pub struct World
{
	pub plant: Plant,
}

impl World
{
	pub fn new(display: &glium::Display) -> World
	{
		let image = image::open("assets/Bonsai_1-500x500.png").unwrap().to_rgba8();
		let image_dimensions = image.dimensions();

		let glium_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

		return World
		{
			plant: Plant
			{
				object_2d: crate::renderer::Object2D
				{
					vertex_buffer: glium::VertexBuffer::new(
						display,
						&[
							crate::renderer::Vertex2D { position: [-0.28, -0.5], texture_coords: [0.0, 0.0] },
							crate::renderer::Vertex2D { position: [ 0.28, -0.5], texture_coords: [1.0, 0.0] },
							crate::renderer::Vertex2D { position: [ 0.28,  0.5], texture_coords: [1.0, 1.0] },
							crate::renderer::Vertex2D { position: [-0.28,  0.5], texture_coords: [0.0, 1.0] },
						]
					).unwrap(),
					index_buffer: glium::IndexBuffer::new(
						display,
						glium::index::PrimitiveType::TrianglesList,
						&[
							0_u16, 1_u16, 2_u16,
							2_u16, 3_u16, 0_u16,
						]
					).unwrap(),
					texture: glium::texture::CompressedSrgbTexture2d::new(display, glium_image).unwrap(),
					texture_shader_program: glium::program!(
						display,
						140 =>
						{
							vertex  : include_str!("../shaders/460/2D/TextureVertShader.glsl"),
							fragment: include_str!("../shaders/460/2D/TextureFragShader.glsl"),
						}
					).unwrap(),
				}
			}
		};
	}

	pub fn update(&mut self)
	{
		self.plant.update();
	}

	pub fn draw(&self, display: &glium::Display)
	{
		self.plant.draw(display);
	}
}