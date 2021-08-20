pub mod math;
pub mod color;
pub mod platform;

fn search_ewmh_desktop_recursive<'conn>(
	window                     : crate::platform::xcb::Window,
	window_type_atom_id        : crate::platform::xcb::AtomID,
	window_type_desktop_atom_id: crate::platform::xcb::AtomID,
)
-> Option<crate::platform::xcb::Window>
{
	match window.get_property(window_type_atom_id)
	{
		Ok(crate::platform::xcb::Property{value: crate::platform::xcb::PropertyValue::Atom(atom_value), ..}) =>
		{
			if atom_value == window_type_desktop_atom_id
			{
				return Some(window);
			}
		},
		Err(error) =>
		{
			println!("Error {:?} al buscar propiedad {}, en la ventana [{}]", error, window_type_atom_id, window.id);
		},
		_ => {/* No hacer nada, significa que no tiene esa propiedad*/}
	};

	for child in window.children()
	{
		if let Some(desktop) = search_ewmh_desktop_recursive(child, window_type_atom_id, window_type_desktop_atom_id)
		{
			return Some(desktop);
		}
	}

	return None;
}

fn search_ewmh_desktop(window: crate::platform::xcb::Window) -> Option<crate::platform::xcb::Window>
{
	let window_type_atom_id = match window.screen.client.find_atom_id_by_name("_NET_WM_WINDOW_TYPE")
	{
		Some(atom_id) => atom_id,
		None => return None
	};

	let window_type_desktop_atom_id = match window.screen.client.find_atom_id_by_name("_NET_WM_WINDOW_TYPE_DESKTOP")
	{
		Some(atom_id) => atom_id,
		None => return None
	};

	return search_ewmh_desktop_recursive(window, window_type_atom_id, window_type_desktop_atom_id)
}

fn pick_fbconfig_with_alpha<'client, 'conn>(xlib_client: &'client crate::platform::xlib::Client<'conn>, screen_num: i32)
-> Option<crate::platform::xlib::FbConfigs<'client, 'conn>>
{
	let visual_attribs = [
		x11::glx::GLX_X_RENDERABLE , 1,
		x11::glx::GLX_DRAWABLE_TYPE, x11::glx::GLX_WINDOW_BIT,
		x11::glx::GLX_RENDER_TYPE  , x11::glx::GLX_RGBA_BIT,
		x11::glx::GLX_X_VISUAL_TYPE, x11::glx::GLX_TRUE_COLOR,
		x11::glx::GLX_RED_SIZE     , 8,
		x11::glx::GLX_GREEN_SIZE   , 8,
		x11::glx::GLX_BLUE_SIZE    , 8,
		x11::glx::GLX_ALPHA_SIZE   , 8,
		x11::glx::GLX_DEPTH_SIZE   , 16,
		x11::glx::GLX_STENCIL_SIZE , 8,
		x11::glx::GLX_DOUBLEBUFFER , 1,
		0
	];

	for fb_configs in xlib_client.search_fb_configs(screen_num, &visual_attribs)
	{
		{
			let visual_info = fb_configs.glx_visual_info();

			if visual_info.render_pict_format().alpha_mask() == 0
			{
				continue;
			}
		}

		return Some(fb_configs);
	}

	return None;
}

fn setup_window<'conn>(
	screen     : &'conn crate::platform::xcb::Screen<'conn>,
	xlib_client: &crate::platform::xlib::Client,
	visual_info: &crate::platform::xlib::VisualInfo,
)
-> crate::platform::xcb::Window<'conn>
{
	let root_window = screen.root_window();

	let ewmh_desktop = search_ewmh_desktop(screen.root_window()).unwrap();

	let geomerty = ewmh_desktop.geometry();

	let colormap = crate::platform::xcb::ColorMap::create(&root_window, visual_info.id());

	let window = root_window.create_child_window(
		geomerty,
		visual_info.depth(),
		Some(&colormap),
		Some(0),
		Some(screen.get_white_pixel()),
		Some(visual_info.id())
	).unwrap();

	//NOTE(fpalacios) Se usa _NET_WM_WINDOW_TYPE_UTILITY para que el Desktop Enviroment no la ponga en la barra de tareas
	window.set_property(
		&crate::platform::xcb::Property
		{
			key  : screen.client.find_atom_id_by_name("_NET_WM_WINDOW_TYPE").unwrap(),
			value: crate::platform::xcb::PropertyValue::Atom(screen.client.find_atom_id_by_name("_NET_WM_WINDOW_TYPE_UTILITY").unwrap())
		}
	);
	screen.client.send_message(
		&root_window,
		crate::platform::xcb::Event::ClientMessageEvent
		{
			window: window.id,
			event_type: screen.client.find_atom_id_by_name("_NET_WM_STATE").unwrap(),
			data:
			[
				1,
				//NOTE(fpalacios): Se usa _NET_WM_STATE_BELOW para que esté abajo de todas las ventanas pero arriba del escritorio real
				screen.client.find_atom_id_by_name("_NET_WM_STATE_BELOW").unwrap(),

				//NOTE(fpalacios): Se usa _NET_WM_STATE_FULLSCREEEN para que use todo el tamaño de la pantalla
				screen.client.find_atom_id_by_name("_NET_WM_STATE_FULLSCREEN").unwrap(),
				0,
				0,
			],
		}
	);

	xlib_client.sync();

	let atom_wm_protocols = screen.client.find_atom_id_by_name("WM_PROTOCOLS").expect("Could not load WM_PROTOCOLS atom");
	let atom_wm_delete_window = screen.client.find_atom_id_by_name("WM_DELETE_WINDOW").expect("Could not load WM_DELETE_WINDOW atom");

	window.set_property(
		&crate::platform::xcb::Property
		{
			key: atom_wm_protocols,
			value: crate::platform::xcb::PropertyValue::Atom(atom_wm_delete_window)
		}
	);

	return window;
}

fn main()
{
	let xcb_client = crate::platform::xcb::Client::with_xlib_display();
	let xlib_client = crate::platform::xlib::Client::from_xcb_client(&xcb_client);
	let preferred_screen = xcb_client.preferred_screen();

	let fb_configs = pick_fbconfig_with_alpha(&xlib_client, xcb_client.preferred_screen_id)
		.expect("No se pudo encontrar un FbConfigs que tenga un alpha_mask");

	let visual_info = fb_configs.glx_visual_info();

	let window = setup_window(&preferred_screen, &xlib_client, &visual_info);

	let glx_context = xlib_client.create_glx_context(&window, &fb_configs);
	let opengl_context = &glx_context.opengl_context;

	let vert_shader = crate::platform::opengl::VertShader::new(&opengl_context, dbg!(include_str!("../shaders/VertShader.glsl")))
		.expect("No se pudo compilar el Vertex Shader");
	let frag_shader = crate::platform::opengl::FragShader::new(&opengl_context, include_str!("../shaders/FragShader.glsl"))
		.expect("No se pudo compilar el Fragment Shader");

	let shader_program = crate::platform::opengl::ShaderProgram::new(&opengl_context, &vert_shader, &frag_shader);

	let triangle = crate::platform::opengl::Polygon::new(
		&opengl_context,
		&crate::math::Shape
		{
			center : crate::math::P3(crate::math::V3 { x: 0_f32, y: 0_f32, z: 0_f32 }),
			vertexs: vec!
			[
				crate::math::P3(crate::math::V3 { x: -0.5_f32, y: 0.0_f32, z: 0_f32 }),
				crate::math::P3(crate::math::V3 { x:  0.0_f32, y: 0.5_f32, z: 0_f32 }),
				crate::math::P3(crate::math::V3 { x:  0.5_f32, y: 0.0_f32, z: 0_f32 }),
			],
		}
	);

	loop
	{
		while let Some(event) = xcb_client.poll_events()
		{
			match event
			{
				crate::platform::xcb::Event::ExposedEvent =>
				{
					println!("Exposed");
				},
				crate::platform::xcb::Event::KeyEvent(crate::platform::xcb::KeyEvent::KeyPress) =>
				{
					println!("keypress");
				},
				crate::platform::xcb::Event::KeyEvent(crate::platform::xcb::KeyEvent::KeyReleased) =>
				{
					println!("keyreleased");
				},
				_ => {}
			}
		}

		let glx_renderer = glx_context.renderer();
		let opengl_renderer = glx_renderer.opengl_renderer();

		opengl_renderer.clear(&crate::color::RgbaColor { r: 0.5_f32, g: 0.5_f32, b: 1_f32, a: 0.5_f32 });
		opengl_renderer.draw_polygon(&triangle);

		xcb_client.flush().unwrap();
		std::thread::sleep(std::time::Duration::from_millis(36));
	}
}