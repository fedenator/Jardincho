use std::convert::TryInto;

//TODO(fpalacios): Mover esto a un closure
pub unsafe fn check_gl_error()
{
	let err = gl::GetError();
	if err != gl::NO_ERROR
	{
		eprintln!("Got gl error {}", err);
	}
}

/*--------------------------------- Polygon Shader -------------------------------------*/
//TODO(fpalacios): Mover esto a otro archivo
pub struct Polygon<'context>
{
	vertex_buffer: VertexBuffer,
	vertexs_count: i32,
	context      : &'context OpenGlContext,
}

impl<'context> Polygon<'context>
{
	pub fn new(context: &'context OpenGlContext, shape: &crate::math::Shape) -> Polygon<'context>
	{
		let polygon = Polygon
		{
			vertex_buffer: VertexBuffer::new(),
			vertexs_count: shape.vertexs.len().try_into().unwrap(),
			context,
		};

		polygon.update_data(&shape);

		return polygon;
	}

	pub fn update_data(&self, shape: &crate::math::Shape)
	{
		let mut data = Vec::new();

		for crate::math::P3(vertex) in &shape.vertexs
		{
			data.push(vertex.x);
			data.push(vertex.y);
			data.push(vertex.z);
		}

		unsafe
		{
			gl::BufferData(
				gl::ARRAY_BUFFER,
				self.vertexs_count as isize * (std::mem::size_of::<f32>() as isize),
				data.as_ptr() as *mut std::os::raw::c_void,
				gl::STATIC_DRAW
			);
		}
	}
}

/*--------------------------------- Vertex Buffer -------------------------------------*/
//TODO(fpalacios): Buscar una forma de ponerle el tiempo de vida del contexto de opengl
pub struct VertexBuffer
{
	pub id        : u32,
	pub next_index: u32,
}

impl VertexBuffer
{
	fn new() -> VertexBuffer
	{
		let mut id = 0;
		unsafe { gl::GenBuffers(1, &mut id) };
		dbg!(id);

		let vertex_buffer = VertexBuffer
		{
			id,
			next_index: 0,
		};

		vertex_buffer.bind();
		unsafe
		{
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0,
				3,
				gl::FLOAT,
				gl::FALSE,
				(std::mem::size_of::<u32>() * 3).try_into().unwrap(),
				0 as *const std::os::raw::c_void
			);
		};

		return vertex_buffer;
	}

	fn bind(&self)
	{
		unsafe { gl::BindVertexArray(self.id) };
	}
}

impl Drop for VertexBuffer
{
	fn drop(&mut self)
	{
		unsafe { gl::DeleteBuffers(self.next_index.try_into().unwrap(), (&mut self.id) as *mut u32) };
	}
}

/*--------------------------------- Shaders -------------------------------------*/
fn compile_shader(src: &str, shader_type: gl::types::GLenum) -> Result<u32, String>
{
	unsafe
	{
		let shader_id = gl::CreateShader(shader_type);

		if shader_id == 0
		{
			return Err(String::from("Error al generar el id del shader"));
		}

		let c_str = std::ffi::CString::new(src.as_bytes()).unwrap();

		gl::ShaderSource(shader_id, 1, &c_str.as_ptr(), std::ptr::null());
		gl::CompileShader(shader_id);

		let mut status = gl::FALSE as gl::types::GLint;
		gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut status);

		if status == gl::FALSE as i32
		{
			// let mut len = 0;
			// gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut len);
			let mut buf = Vec::with_capacity(512);

			gl::GetShaderInfoLog(
				shader_id,
				buf.capacity().try_into().unwrap(),
				std::ptr::null_mut(),
				buf.as_mut_ptr() as *mut gl::types::GLchar,
			);


			return Err(String::from_utf8(buf).expect("ShaderInfoLog not valid utf8"));
		}

		return Ok(shader_id);
	}
}

pub struct FragShader<'context>
{
	pub id     : u32,
	pub context: &'context OpenGlContext,
}

impl<'context> FragShader<'context>
{
	pub fn new(context: &'context OpenGlContext, src: &str) -> Result<FragShader<'context>, String>
	{
		return Ok(
			FragShader
			{
				id: compile_shader(src, gl::FRAGMENT_SHADER)?,
				context,
			}
		);
	}
}

impl<'context> Drop for FragShader<'context>
{
	fn drop(&mut self)
	{
		unsafe { gl::DeleteShader(self.id) };
	}
}

pub struct VertShader<'context>
{
	pub context: &'context OpenGlContext,
	pub id     : u32,
}

impl<'context> VertShader<'context>
{
	pub fn new(context: &'context OpenGlContext, src: &str) -> Result<VertShader<'context>, String>
	{
		return Ok(
			VertShader
			{
				id: compile_shader(src, gl::VERTEX_SHADER)?,
				context,
			}
		);
	}
}

impl<'context> Drop for VertShader<'context>
{
	fn drop(&mut self)
	{
		unsafe { gl::DeleteShader(self.id) };
	}
}

pub struct ShaderProgram<'vert_shader, 'frag_shader, 'context>
{
	id         : u32,
	context    : &'context OpenGlContext,
	vert_shader: &'vert_shader VertShader<'context>,
	frag_shader: &'frag_shader FragShader<'context>,
}

impl<'vert_shader, 'frag_shader, 'context> ShaderProgram<'vert_shader, 'frag_shader, 'context>
{
	pub fn new(
		context    : &'context OpenGlContext,
		vert_shader: &'vert_shader VertShader<'context>,
		frag_shader: &'frag_shader FragShader<'context>
	)
	-> Result<ShaderProgram<'vert_shader, 'frag_shader, 'context>, String>
	{
		unsafe
		{
			let program = gl::CreateProgram();

			gl::AttachShader(program, vert_shader.id);
			gl::AttachShader(program, frag_shader.id);
			gl::LinkProgram(program);

			let mut status = gl::FALSE as gl::types::GLint;
			gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

			if status != (gl::TRUE as gl::types::GLint)
			{
				let mut len: gl::types::GLint = 0;
				gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
				let mut buf = Vec::with_capacity(len as usize);
				// buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
				gl::GetProgramInfoLog(
					program,
					len,
					std::ptr::null_mut(),
					buf.as_mut_ptr() as *mut gl::types::GLchar,
				);

				return Err(String::from_utf8(buf).expect("ProgramInfoLog not valid utf8"));
			}

			return Ok(
				ShaderProgram
				{
					id: program,
					context,
					vert_shader,
					frag_shader,
				}
			);
		}
	}
}

impl<'vert_shader, 'frag_shader, 'context> Drop for ShaderProgram<'vert_shader, 'frag_shader, 'context>
{
	fn drop(&mut self)
	{
		unsafe { gl::DeleteProgram(self.id) };
	}
}

/*--------------------------------- OpenGL Context -------------------------------------*/
pub struct OpenGlContext
{
}

impl OpenGlContext
{
	pub fn new() -> OpenGlContext
	{
		return OpenGlContext
		{
		}
	}
}

/*--------------------------------- OpenGL Renderer -------------------------------------*/
pub struct OpenGlRenderer<'context>
{
	context: &'context OpenGlContext,
}

impl<'context> OpenGlRenderer<'context>
{
	pub fn new(context: &'context OpenGlContext) -> OpenGlRenderer<'context>
	{
		return OpenGlRenderer
		{
			context,
		};
	}

	pub fn clear(&self, color: &crate::color::RgbaColor)
	{
		unsafe
		{
			gl::ClearColor(color.r, color.g, color.b, color.a);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
		}
	}

	pub fn draw_polygon(&self, polygon: &Polygon)
	{
		unsafe
		{
			polygon.vertex_buffer.bind();
			gl::DrawArrays(gl::TRIANGLES, 0, polygon.vertexs_count);
		}
	}
}

impl<'context> Drop for OpenGlRenderer<'context>
{
	fn drop(&mut self)
	{
		unsafe
		{
			gl::Flush();
		}
	}
}