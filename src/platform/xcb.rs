pub type AtomID            = xcb::Atom;
pub type Color             = u32;
pub type ScreenID          = i32;
pub type WindowID          = xcb::Window;
pub type DrawableID        = xcb::Window;
pub type GraphicsContextID = xcb::Atom;
pub type EventKeyID        = xcb::EventMask;
pub type ColorMapID        = xcb::Atom;
pub type PixMapID          = xcb::Atom;
pub type VisualID          = xcb::Atom;

#[derive(Debug)]
pub struct Error
{
	pub error_code: u8
}

pub enum Event
{
	KeyEvent(KeyEvent),
	ExposedEvent,
	//TODO(fpalacios): Mover los datos de ClientMessageEvent a su propia struct (como en KeyEvent)
	ClientMessageEvent
	{
		window       : WindowID,
		event_type   : AtomID,
		data         : [u32; 5],
	},
	UnknownEvent(xcb::EventMask, xcb::Event<xcb::ffi::xcb_generic_event_t>)
}

#[derive(Debug)]
pub enum KeyEvent
{
	KeyPress,
	KeyReleased,
}

#[derive(Debug)]
pub struct Property
{
	pub key  : AtomID,
	pub value: PropertyValue,
}

#[derive(Debug)]
pub enum PropertyValue
{
	String(String),
	I32(i32),
	U32(u32),
	None,
	Atom(AtomID),
	UnknownAtom(AtomID),
}

impl PropertyValue
{
	pub fn get_type_atom_id(&self) -> AtomID
	{
		return match self
		{
			PropertyValue::String(_) => xcb::ATOM_STRING,
			PropertyValue::I32(_) => xcb::ATOM_INTEGER,
			PropertyValue::U32(_) => xcb::ATOM_CARDINAL,
			PropertyValue::Atom(_) => xcb::ATOM_ATOM,
			PropertyValue::UnknownAtom(atom_id) => atom_id.clone(),
			PropertyValue::None => xcb::ATOM_NONE
		};
	}
}

pub struct Client
{
	pub conn               : xcb::Connection,
	pub preferred_screen_id: ScreenID,
}

impl Client
{
	pub fn new() -> Client
	{
		let (conn, preferred_screen_id) = xcb::Connection::connect(None).unwrap();

		return Client
		{
			conn,
			preferred_screen_id
		};
	}

	pub fn with_xlib_display() -> Client
	{
		let (conn, preferred_screen_id) = xcb::Connection::connect_with_xlib_display().unwrap();
		conn.set_event_queue_owner(xcb::EventQueueOwner::Xcb);

		return Client
		{
			conn,
			preferred_screen_id
		};
	}

	pub fn preferred_screen(&self) -> Screen
	{
		let screen = self.conn.get_setup().roots().nth(self.preferred_screen_id as usize).unwrap();

		return Screen
		{
			id        : self.preferred_screen_id,
			client    : self,
			xcb_screen: screen,
		};
	}

	pub fn find_atom_id_by_name(&self, name: &str) -> Option<AtomID>
	{
		let atom_id = xcb::intern_atom(&self.conn, false, name).get_reply().unwrap().atom();
		return if atom_id == xcb::ATOM_NONE { None } else { Some(atom_id) };
	}

	pub fn find_atom_name(&self, atom_id: AtomID) -> String
	{
		return xcb::get_atom_name(&self.conn, atom_id).get_reply().unwrap().name().to_owned();
	}

	pub fn poll_events(&self) -> Option<Event>
	{
		let event = match self.conn.poll_for_event()
		{
			Some(event) => event,
			None => return None,
		};

		match event.response_type() & !0x80
		{
			xcb::EXPOSE => return Some(Event::ExposedEvent),
			xcb::KEY_PRESS => return Some(Event::KeyEvent(KeyEvent::KeyPress)),
			xcb::KEY_RELEASE => return Some(Event::KeyEvent(KeyEvent::KeyReleased)),
			event =>
			{
				println!("UNKOWN EVENT {:?}", event);
				return None;
			}
		};
	}

	pub fn send_message(&self, destination: &Window, event: Event)
	{
		match event
		{
			Event::ClientMessageEvent {window, event_type, data , ..} =>
			{
				let message_data = xcb::ffi::xproto::xcb_client_message_data_t::from_data32(data);

				let event = xcb::Event::<xcb::ffi::xproto::xcb_client_message_event_t>::new(
					32,
					window,
					event_type,
					message_data
				);

				xcb::send_event_checked(
					&self.conn,
					false,
					destination.id,
					xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
					&event
				).request_check().unwrap();
			}
			_ =>
			{
				//TODO(fpalacios): Ver que hacer acá
			}
		};

		self.flush().unwrap();
	}

	pub fn flush(&self) -> Result<(), ()>
	{
		return if self.conn.flush() { Ok(()) } else { Err(()) };
	}

	fn generate_id(&self) -> u32
	{
		return self.conn.generate_id();
	}
}

pub struct Screen<'conn>
{
	pub id        : ScreenID,
	pub client    : &'conn Client,
	pub xcb_screen: xcb::Screen<'conn>,
}

impl<'conn> Screen<'conn>
{
	pub fn root_window(&self) -> Window
	{
		return Window
		{
			screen: self,
			id: self.xcb_screen.root()
		};
	}

	pub fn get_black_pixel(&self) -> Color
	{
		return self.xcb_screen.black_pixel();
	}
	pub fn get_white_pixel(&self) -> Color
	{
		return self.xcb_screen.white_pixel();
	}
}

pub struct Window<'conn>
{
	pub screen: &'conn Screen<'conn>,
	pub id: WindowID,
}

impl<'conn> Window<'conn>
{
	pub fn children(&self) -> Vec<Window<'conn>>
	{
		let tree = xcb::query_tree(&self.screen.client.conn, self.id).get_reply().unwrap();
		let children = tree.children();

		let mut result = Vec::with_capacity(children.len());

		for child in children
		{
			result.push(Window { screen: self.screen, id: child.clone()});
		}

		return result;
	}

	pub fn get_property(&self, atom: AtomID) -> Result<Property, Error>
	{
		let property = match xcb::get_property(
			&self.screen.client.conn,
			false,
			self.id,
			atom,
			xcb::ATOM_ANY,
			0,
			1024
		).get_reply()
		{
			Ok(property) => property,
			Err(err)     =>
			{
				return Err(
					Error
					{
						error_code: err.error_code()
					}
				);
			}
		};

		let value = match property.type_()
		{
			xcb::ATOM_STRING   => PropertyValue::String((*String::from_utf8_lossy(property.value::<u8>())).to_owned()),
			xcb::ATOM_INTEGER  => PropertyValue::I32(property.value::<i32>()[0]),
			xcb::ATOM_NONE     => PropertyValue::None,
			xcb::ATOM_ATOM     => PropertyValue::Atom(property.value::<u32>()[0]),
			xcb::ATOM_CARDINAL => PropertyValue::U32(property.value::<u32>()[0]),
			unknown_atom =>
			{
				match self.screen.client.find_atom_name(unknown_atom).as_ref()
				{
					"UTF8_STRING" => PropertyValue::String((*String::from_utf8_lossy(property.value::<u8>())).to_owned()),
					_             => PropertyValue::UnknownAtom(unknown_atom)
				}
			}
		};

		return Ok(Property{ key: atom, value });
	}

	pub fn set_property(&self, property: &Property)
	{
		let atom_type = property.value.get_type_atom_id();

		match &property.value
		{
			PropertyValue::String(val) =>
			{
				xcb::change_property(
					&self.screen.client.conn,
					xcb::PROP_MODE_REPLACE as u8,
					self.id,
					property.key,
					atom_type,
					8,
					val.as_bytes()
				);
			},
			PropertyValue::Atom(val) =>
			{
				xcb::change_property(
					&self.screen.client.conn,
					xcb::PROP_MODE_REPLACE as u8,
					self.id,
					property.key,
					atom_type,
					32,
					&[val.clone()]
				);
			},
			PropertyValue::I32(val) =>
			{
				xcb::change_property(
					&self.screen.client.conn,
					xcb::PROP_MODE_REPLACE as u8,
					self.id,
					property.key,
					atom_type,
					32,
					&[val.clone()]
				);
			},
			PropertyValue::U32(val) =>
			{
				xcb::change_property(
					&self.screen.client.conn,
					xcb::PROP_MODE_REPLACE as u8,
					self.id,
					property.key,
					atom_type,
					32,
					&[val.clone()]
				);
			},
			PropertyValue::None =>
			{
				xcb::change_property(
					&self.screen.client.conn,
					xcb::PROP_MODE_REPLACE as u8,
					self.id,
					property.key,
					atom_type,
					32,
					&[xcb::ATOM_NONE]
				);
			},
			PropertyValue::UnknownAtom(_) =>
			{
				//TODO(fpalacios): Que hacemo acá?
				panic!("Que hacemo acá?");
			},
		};
	}

	pub fn geometry(&self) -> (i16, i16, u16, u16)
	{
		let geometry = match xcb::get_geometry(&self.screen.client.conn, self.id).get_reply()
		{
			Ok(geomerty) => geomerty,
			Err(error) =>
			{
				println!("Error al obtener la geometria. Error code [{}]", error.error_code());
				panic!();
			}
		};

		return (geometry.x(), geometry.y(), geometry.width(), geometry.height());
	}

	pub fn map(&self)
	{
		xcb::map_window(&self.screen.client.conn, self.id);
		self.screen.client.flush().unwrap();
	}

	pub fn create_child_window(
		&self,
		(x, y, width, height): (i16, i16, u16, u16),
		depth                : u8,
		colormap             : Option<&ColorMap>,
		background_pixel     : Option<u32>,
		border_pixel         : Option<u32>,
		visual_id            : Option<VisualID>
	)
	-> Result<Window<'conn>, Error>
	{
		let child_id = self.screen.client.generate_id();

		let mut window_attributes = vec![
			(
				xcb::CW_EVENT_MASK,
				xcb::GC_GRAPHICS_EXPOSURES | xcb::EVENT_MASK_KEY_PRESS
			)
		];

		if let Some(colormap) = colormap
		{
			window_attributes.push((xcb::CW_COLORMAP, colormap.id));
		}

		if let Some(background_pixel) = background_pixel
		{
			window_attributes.push((xcb::CW_BACK_PIXEL, background_pixel))
		}

		if let Some(border_pixel) = border_pixel
		{
			window_attributes.push((xcb::CW_BORDER_PIXEL, border_pixel));
		}

		let visual_id = match visual_id
		{
			Some(visual_id) => visual_id,
			None => self.screen.xcb_screen.root_visual()
		};

		if let Err(e) = xcb::create_window_checked(
			&self.screen.client.conn,
			depth,
			child_id,
			self.id,
			x,
			y,
			width,
			height,
			1,
			xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
			visual_id,
			&window_attributes
		).request_check()
		{
			return Err(Error{error_code: e.error_code()})
		};

		self.screen.client.flush().unwrap();

		let window = Window
		{
			screen: self.screen,
			id    : child_id,
		};

		window.map();
		return Ok(window);
	}
}

pub struct GraphicsContext<'conn>
{
	id    : GraphicsContextID,
	client: &'conn Client
}

impl<'conn> GraphicsContext<'conn>
{
	pub fn generate(window: &'conn Window, foreground: Color, background: Color) -> GraphicsContext<'conn>
	{
		let id = window.screen.client.generate_id();

		xcb::create_gc_checked(
			&window.screen.client.conn,
			id,
			window.id,
			&[
				(xcb::GC_FOREGROUND, foreground),
				(xcb::GC_BACKGROUND, background),
				(xcb::GC_LINE_WIDTH, 1),
				(xcb::GC_LINE_STYLE, xcb::LINE_STYLE_SOLID),
				(xcb::GC_GRAPHICS_EXPOSURES, 0)
			]
		).request_check().unwrap();

		return GraphicsContext
		{
			client: window.screen.client,
			id,
		};
	}


	pub fn draw_rects(&self, window: &'conn Window, rectangles: &[xcb::Rectangle])
	{
		xcb::poly_rectangle(&self.client.conn, window.id, self.id, &rectangles);
	}

	pub fn fill_rects(&self, window: &'conn Window, rectangles: &[xcb::Rectangle])
	{
		xcb::poly_fill_rectangle(&self.client.conn, window.id, self.id, &rectangles);
	}

	//TODO(fpalacios): Ver como pintar una imagen en xcb puro
	pub fn draw_image(
		&self,
		drawable: DrawableID,
		_image: &image::RgbaImage,
		(x, y): (i16, i16),
	)
	{

		//NOTE(fpalacios): Generar y llear este array es para una prueba puntal de ahora
		let mut a = [0_u8; 100 * 100 * 4];

		for i in (0 .. 100).step_by(4)
		{
			a[i + 0] = 0x00;
			a[i + 1] = 0xFF;
			a[i + 2] = 0xFF;
			a[i + 3] = 0x00;
		}

		if let Err(e) = xcb::put_image_checked(
			&self.client.conn,
			xcb::IMAGE_FORMAT_Z_PIXMAP as u8,
			drawable,
			self.id,
			100,
			100,
			x,
			y,
			0,
			32,
			&a,
		).request_check()
		{
			println!("Error al pintar la imagen {:?}", e);
		}
	}

	pub fn clear_window(&self, window: &'conn Window)
	{
		let (x, y, width, height) = window.geometry();
		xcb::clear_area(&self.client.conn, true, window.id, x, y, width, height);
	}

	pub fn clear_area(&self, window: &'conn Window, (x, y, width, height): (i16, i16, u16, u16))
	{
		xcb::clear_area(&self.client.conn, true, window.id, x, y, width, height);
	}
}

pub struct ColorMap<'conn>
{
	pub client: &'conn Client,
	pub id    : ColorMapID
}

impl<'conn> ColorMap<'conn>
{
	pub fn create(window: &'conn Window, visual: VisualID) -> ColorMap<'conn>
	{
		let client = window.screen.client;
		let id = client.generate_id();

		xcb::create_colormap_checked(
			&client.conn,
			xcb::COLORMAP_ALLOC_NONE as u8,
			id,
			window.id,
			visual
		).request_check().unwrap();

		return ColorMap
		{
			client,
			id
		};
	}
}

pub struct PixMap<'conn>
{
	pub client: &'conn Client,
	pub id    : PixMapID,
	pub width : u16,
	pub height: u16
}

impl<'conn> PixMap<'conn>
{
	pub fn create(screen: &'conn Screen, drawable: DrawableID, width: u16, height: u16) -> PixMap<'conn>
	{
		let client = screen.client;
		let id = client.generate_id();

		xcb::create_pixmap_checked(
			&client.conn,
			32,
			id,
			drawable,
			width,
			height
		).request_check().unwrap();

		return PixMap
		{
			client,
			id,
			width,
			height
		};
	}
}