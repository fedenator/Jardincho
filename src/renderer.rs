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

pub struct Object2D
{
	pub vertex_buffer         : glium::VertexBuffer<Vertex2D>,
	pub index_buffer          : glium::IndexBuffer<u16>,
	pub texture               : glium::texture::CompressedSrgbTexture2d,
	pub texture_shader_program: glium::Program,
}

pub struct Object3D
{
	pub rotation_matrix    : nalgebra::Matrix4<f32>,
	pub translation_vector : nalgebra::Vector3<f32>,
	pub vertex_buffer      : glium::VertexBuffer<Vertex3D>,
	pub index_buffer       : glium::IndexBuffer<u16>,
	pub fill_shader_program: glium::Program,
	pub wire_shader_program: glium::Program,
}

pub fn draw_object_2d(display: &glium::Display, object: &Object2D)
{
	let uniforms = glium::uniform!
	{
		texture2d: &object.texture
	};

	let mut target = display.draw();
	glium::Surface::clear_color(&mut target, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);

	let draw_parameters = glium::DrawParameters::default();

	glium::Surface::draw(
		&mut target,
		&object.vertex_buffer,
		&object.index_buffer,
		&object.texture_shader_program,
		&uniforms,
		&draw_parameters
	).unwrap();

	target.finish().unwrap();
}

pub fn draw_object_3d(display: &glium::Display, object: &Object3D)
{
	let uniforms = glium::uniform!
	{
		rotation:
		[
			[object.rotation_matrix[ 0], object.rotation_matrix[ 1], object.rotation_matrix[ 2], object.rotation_matrix[ 3]],
			[object.rotation_matrix[ 4], object.rotation_matrix[ 5], object.rotation_matrix[ 6], object.rotation_matrix[ 7]],
			[object.rotation_matrix[ 8], object.rotation_matrix[ 9], object.rotation_matrix[10], object.rotation_matrix[11]],
			[object.rotation_matrix[12], object.rotation_matrix[13], object.rotation_matrix[14], object.rotation_matrix[15]],
		],
		translation: [object.translation_vector.x, object.translation_vector.y, object.translation_vector.z, 0_f32]
	};

	let mut target = display.draw();
	glium::Surface::clear_color(&mut target, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);

	let mut draw_parameters = glium::DrawParameters::default();

	draw_parameters.polygon_mode = glium::PolygonMode::Fill;

	glium::Surface::draw(
		&mut target,
		&object.vertex_buffer,
		&object.index_buffer,
		&object.fill_shader_program,
		&uniforms,
		&draw_parameters
	).unwrap();

	draw_parameters.polygon_mode = glium::PolygonMode::Line;

	glium::Surface::draw(
		&mut target,
		&object.vertex_buffer,
		&object.index_buffer,
		&object.wire_shader_program,
		&uniforms,
		&draw_parameters
	).unwrap();

	target.finish().unwrap();
}