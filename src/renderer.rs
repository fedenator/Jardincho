glium::implement_vertex!(Vertex, position, color);
#[derive(Copy, Clone)]
pub struct Vertex
{
	pub position: [f32; 3],
	pub color   : [f32; 3],
}

pub struct Object
{
	pub rotation_matrix    : nalgebra::Matrix4<f32>,
	pub translation_vector : nalgebra::Vector3<f32>,
	pub vertex_buffer      : glium::VertexBuffer<Vertex>,
	pub index_buffer       : glium::IndexBuffer<u16>,
	pub fill_shader_program: glium::Program,
	pub wire_shader_program: glium::Program,
}

pub fn draw(display: &glium::Display, object: &Object)
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