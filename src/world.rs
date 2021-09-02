use glium::program;
pub struct Plant
{
	animation: crate::renderer::Animation2D,
}

impl Plant
{
	fn update(&mut self, delta: &std::time::Duration)
	{
		self.animation.update(delta);
	}

	pub fn draw(&self, display: &glium::Display)
	{
		crate::renderer::draw_animation_2d(&display, &self.animation);
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
		let image1 = image::open("assets/Bonsai_1-500x500.png").unwrap().to_rgba8();
		let image2 = image::open("assets/Bonsai_2-500x500.png").unwrap().to_rgba8();
		let image3 = image::open("assets/Bonsai_3-500x500.png").unwrap().to_rgba8();

		let image_dimensions_1 = image1.dimensions();
		let image_dimensions_2 = image2.dimensions();
		let image_dimensions_3 = image3.dimensions();

		let glium_image1 = glium::texture::RawImage2d::from_raw_rgba_reversed(&image1.into_raw(), image_dimensions_1);
		let glium_image2 = glium::texture::RawImage2d::from_raw_rgba_reversed(&image2.into_raw(), image_dimensions_2);
		let glium_image3 = glium::texture::RawImage2d::from_raw_rgba_reversed(&image3.into_raw(), image_dimensions_3);

		let texture1 = std::rc::Rc::new(glium::texture::CompressedSrgbTexture2d::new(display, glium_image1).unwrap());
		let texture2 = std::rc::Rc::new(glium::texture::CompressedSrgbTexture2d::new(display, glium_image2).unwrap());
		let texture3 = std::rc::Rc::new(glium::texture::CompressedSrgbTexture2d::new(display, glium_image3).unwrap());

		let root_steps_chain = std::rc::Rc::new(
			crate::renderer::Animation2DStep
			{
				texture  : texture1.clone(),
				duration : std::time::Duration::from_millis(1000),
				next_step: Some(std::rc::Rc::new(
					crate::renderer::Animation2DStep
					{
						texture  : texture2,
						duration : std::time::Duration::from_millis(1000),
						next_step: Some(std::rc::Rc::new(
							crate::renderer::Animation2DStep
							{
								texture  : texture3,
								duration : std::time::Duration::from_millis(1000),
								next_step: None,
							}
						)),
					}
				)),
			}
		);

		return World
		{
			plant: Plant
			{
				animation: crate::renderer::Animation2D
				{
					model: crate::renderer::Model2D
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
						texture: texture1,
						texture_shader_program: glium::program!(
							display,
							140 =>
							{
								vertex  : include_str!("../shaders/460/2D/TextureVertShader.glsl"),
								fragment: include_str!("../shaders/460/2D/TextureFragShader.glsl"),
							}
						).unwrap(),
					},
					timer: std::time::Duration::from_millis(0),
					root_steps_chain: root_steps_chain.clone(),
					current_step: root_steps_chain,
				}
			}

		};
	}

	pub fn update(&mut self, delta: &std::time::Duration)
	{
		self.plant.update(delta);
	}

	pub fn draw(&self, display: &glium::Display)
	{
		self.plant.draw(display);
	}
}