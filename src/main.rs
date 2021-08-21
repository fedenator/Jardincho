pub mod math;
pub mod color;
pub mod platform;

use glium::{glutin::platform::unix::WindowExtUnix, program};

glium::implement_vertex!(Vertex, position, color);
#[derive(Copy, Clone)]
struct Vertex
{
	position: [f32; 2],
	color   : [f32; 3],
}

fn setup_xcb_window(xcb_conn: &xcb::Connection, window_context: &glium::glutin::window::Window)
{
	let xcb_client = crate::platform::xcb::Client::new(&xcb_conn);

	let xcb_screen = crate::platform::xcb::Screen::from_id(
		&xcb_client,
		window_context.xlib_screen_id().unwrap() as crate::platform::xcb::ScreenID
	).unwrap();

	let xcb_window = crate::platform::xcb::Window
	{
		screen: &xcb_screen,
		id    : window_context.xlib_window().unwrap() as crate::platform::xcb::WindowID,
	};

	xcb_window.set_property(
		&crate::platform::xcb::Property
		{
			key  : xcb_client.find_atom_id_by_name("_NET_WM_WINDOW_TYPE").unwrap(),
			value: crate::platform::xcb::PropertyValue::Atom(xcb_client.find_atom_id_by_name("_NET_WM_WINDOW_TYPE_UTILITY").unwrap())
		}
	);
	xcb_client.send_message(
		&xcb_screen.root_window(),
		crate::platform::xcb::Event::ClientMessageEvent
		{
			window: xcb_window.id,
			event_type: xcb_client.find_atom_id_by_name("_NET_WM_STATE").unwrap(),
			data:
			[
				1,
				//NOTE(fpalacios): Se usa _NET_WM_STATE_BELOW para que esté abajo de todas las ventanas pero arriba del escritorio real
				xcb_client.find_atom_id_by_name("_NET_WM_STATE_BELOW").unwrap(),

				//NOTE(fpalacios): Se usa _NET_WM_STATE_FULLSCREEEN para que use todo el tamaño de la pantalla
				xcb_client.find_atom_id_by_name("_NET_WM_STATE_FULLSCREEN").unwrap(),
				0,
				0,
			],
		}
	);
}

fn setup_window(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> glium::Display
{
	let wb = glium::glutin::window::WindowBuilder::new()
		.with_transparent(true);

	let cb = glium::glutin::ContextBuilder::new();

	let display = glium::Display::new(wb, cb, &event_loop).unwrap();
	{
		let gl_window = display.gl_window();
		let window_context = gl_window.window();

		//NOTE(fpalacios): Hace el setup especifico de cada plataforma
		if let Some(xcb_conn) = window_context.xcb_connection()
		{
			let xcb_conn = unsafe { xcb::Connection::from_raw_conn(xcb_conn as *mut xcb::ffi::xcb_connection_t) };
			setup_xcb_window(&xcb_conn, window_context);

			//NOTE(fpalacios): Infame hack para que no se llame al destructor de xcb::Connection y le mate la conexion que en realidad no es nuestra :V
			xcb_conn.into_raw_conn();
		}
	}

	return display;
}

fn draw(
	display      : &glium::Display,
	program      : &glium::Program,
	vertex_buffer: &glium::VertexBuffer<Vertex>,
	index_buffer : &glium::IndexBuffer<u16>
)
{
	// building the uniforms
	let uniforms = glium::uniform!
	{
		matrix:
		[
			[1.0_f32, 0.0_f32, 0.0_f32, 0.0_f32],
			[0.0_f32, 1.0_f32, 0.0_f32, 0.0_f32],
			[0.0_f32, 0.0_f32, 1.0_f32, 0.0_f32],
			[0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32]
		]
	};

	// drawing a frame
	let mut target = display.draw();
	glium::Surface::clear_color(&mut target, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
	glium::Surface::draw(&mut target, vertex_buffer, index_buffer, &program, &uniforms, &Default::default()).unwrap();
	target.finish().unwrap();
}


fn main()
{
	let event_loop = glium::glutin::event_loop::EventLoop::new();
	let display = setup_window(&event_loop);

	let vertex_buffer = glium::VertexBuffer::new(
		&display,
		&[
			Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
			Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
			Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
		]
	).unwrap();

	let index_buffer = glium::IndexBuffer::new(
		&display,
		glium::index::PrimitiveType::TrianglesList,
		&[0_u16, 1_u16, 2_u16]
	).unwrap();

	let program = glium::program!(
		&display,
		140 =>
		{
			vertex  : include_str!("../shaders/140/VertShader.glsl"),
			fragment: include_str!("../shaders/140/FragShader.glsl")
		},
		110 =>
		{
			vertex  : include_str!("../shaders/110/VertShader.glsl"),
			fragment: include_str!("../shaders/110/FragShader.glsl"),
		},
		100 =>
		{
			vertex  : include_str!("../shaders/100/VertShader.glsl"),
			fragment: include_str!("../shaders/100/FragShader.glsl"),
		},
	).unwrap();

	draw(&display, &program, &vertex_buffer, &index_buffer);

	event_loop.run(move |event, _, control_flow|
	{
		*control_flow = match event {
			glium::glutin::event::Event::WindowEvent { event, .. } => match event {
				// Break from the main loop when the window is closed.
				glium::glutin::event::WindowEvent::CloseRequested => glium::glutin::event_loop::ControlFlow::Exit,
				// Redraw the triangle when the window is resized.
				glium::glutin::event::WindowEvent::Resized(..) => {
					draw(&display, &program, &vertex_buffer, &index_buffer);
					glium::glutin::event_loop::ControlFlow::Poll
				},
				_ => glium::glutin::event_loop::ControlFlow::Poll,
			},
			_ => glium::glutin::event_loop::ControlFlow::Poll,
		};
	});
}