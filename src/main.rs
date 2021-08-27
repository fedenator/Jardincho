pub mod world;
pub mod color;
pub mod platform;
pub mod renderer;

use glium::glutin::platform::unix::WindowExtUnix;

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

fn main()
{
	let event_loop = glium::glutin::event_loop::EventLoop::new();
	let display = setup_window(&event_loop);

	let mut world = crate::world::World::new(&display);

	event_loop.run( move |event, _, control_flow|
	{
		*control_flow = match event
		{
			glium::glutin::event::Event::WindowEvent
			{
				event: glium::glutin::event::WindowEvent::CloseRequested,
				..
			} => glium::glutin::event_loop::ControlFlow::Exit,
			_ => glium::glutin::event_loop::ControlFlow::Poll,
		};

		world.update();
		world.draw(&display);

		let next_frame_time = std::time::Instant::now() + std::time::Duration::from_millis(1000000);
		*control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
	});
}