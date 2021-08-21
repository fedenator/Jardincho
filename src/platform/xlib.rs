use std::convert::TryInto;

/*-------------------------- Cargar recursos externos de OpenGL y GLX -----------------*/
//TODO(fpalacios): GLX_FLAG
pub type GlXCreateContextAttribsARBProc = unsafe extern "C" fn (
	dpy          : *mut x11::xlib::Display,
	fbc          : x11::glx::GLXFBConfig,
	share_context: x11::glx::GLXContext,
	direct       : x11::xlib::Bool,
	attribs      : *const std::os::raw::c_int
)
-> x11::glx::GLXContext;

//TODO(fpalacios): GLX_FLAG
unsafe fn load_gl_func(name: &str) -> *mut std::os::raw::c_void
{
	let cname = std::ffi::CString::new(name).unwrap();
	let ptr: *mut std::os::raw::c_void = std::mem::transmute(x11::glx::glXGetProcAddress(cname.as_ptr() as *const u8));
	if ptr.is_null()
	{
		panic!("could not load {}", name);
	}
	return ptr;
}

//TODO(fpalacios): GLX_FLAG
pub static mut CTX_ERROR_OCCURRED: bool = false;

//TODO(fpalacios): GLX_FLAG
pub unsafe extern "C" fn ctx_error_handler(
	_dpy: *mut x11::xlib::Display,
	_ev : *mut x11::xlib::XErrorEvent
)
-> i32
{
	CTX_ERROR_OCCURRED = true;
	return 0;
}

/*---------------------- Wrapper de xlib ------------*/
pub struct RenderPictFormat<'visual_info, 'fb_configs, 'client, 'conn>
{
	pub xrender_pict_format: *mut x11::xrender::XRenderPictFormat,
	pub visual_info        : &'visual_info VisualInfo<'fb_configs, 'client, 'conn>,
}

impl<'visual_info, 'fb_configs, 'client, 'conn> RenderPictFormat<'visual_info, 'fb_configs, 'client, 'conn>
{
	pub fn alpha_mask(&self) -> i16
	{
		return (unsafe {*self.xrender_pict_format}).direct.alphaMask;
	}
}

pub struct VisualInfo<'fb_configs, 'client, 'conn>
{
	pub xvisual_info: *mut x11::xlib::XVisualInfo,
	pub fb_configs  : &'fb_configs FbConfigs<'client, 'conn>,
}

impl<'fb_configs, 'client, 'conn> VisualInfo<'fb_configs, 'client, 'conn>
{
	pub fn render_pict_format(&self) -> RenderPictFormat
	{
		let xrender_pict_format = unsafe
		{
			x11::xrender::XRenderFindVisualFormat(
				self.fb_configs.client.xlib_display,
				(*self.xvisual_info).visual
			)
		};

		return RenderPictFormat
		{
			xrender_pict_format,
			visual_info: self,
		};
	}

	pub fn id(&self) -> crate::platform::xcb::VisualID
	{
		return (unsafe {*self.xvisual_info}).visualid.try_into().unwrap();
	}

	pub fn depth(&self) -> u8
	{
		return (unsafe {*self.xvisual_info}).depth.try_into().unwrap();
	}
}

impl<'fb_configs, 'client, 'conn> Drop for VisualInfo<'fb_configs, 'client, 'conn>
{
	fn drop(&mut self)
	{
		unsafe { x11::xlib::XFree(self.xvisual_info as *mut std::os::raw::c_void) };
	}
}

pub struct FbConfigs<'client, 'conn>
{
	pub glx_fb_config: *mut x11::glx::__GLXFBConfigRec,
	pub client       : &'client Client<'conn>,
}

impl<'client, 'conn> FbConfigs<'client, 'conn>
{
	//TODO(fpalacios): GLX_FLAG
	pub fn glx_visual_info(&self) -> VisualInfo
	{
		return VisualInfo
		{
			xvisual_info: unsafe { x11::glx::glXGetVisualFromFBConfig(self.client.xlib_display, self.glx_fb_config) },
			fb_configs  : &self
		};
	}
}

pub struct FbConfigsIterator<'client, 'conn>
{
	pub size         : isize,
	pub root         : *mut *mut x11::glx::__GLXFBConfigRec,
	pub client       : &'client Client<'conn>,
	pub current_index: isize,
}

impl<'client, 'conn> Iterator for FbConfigsIterator<'client, 'conn>
{
	type Item = FbConfigs<'client, 'conn>;

	fn next(&mut self) -> Option<Self::Item>
	{
		let fb_configs = FbConfigs
		{
			glx_fb_config: unsafe { *self.root.offset(self.current_index) },
			client       : self.client
		};

		self.current_index += 1;

		return Some(fb_configs);
	}
}

impl<'client, 'conn> Drop for FbConfigsIterator<'client, 'conn>
{
	fn drop(&mut self)
	{
		unsafe { x11::xlib::XFree(self.root as *mut std::os::raw::c_void) };
	}
}

pub struct GlxContext<'window, 'client, 'conn>
{
	pub window        : &'window crate::platform::xcb::Window<'conn>,
	pub client        : &'client Client<'conn>,
	pub context_ptr   : *mut x11::glx::__GLXcontextRec,
	pub opengl_context: crate::platform::opengl::OpenGlContext,
}

impl<'window, 'client, 'conn> GlxContext<'window, 'client, 'conn>
{
	pub fn renderer<'context>(&'context self) -> GlxWindowRenderer<'context, 'window, 'client, 'conn>
	{
		return GlxWindowRenderer::new(self);
	}
}

impl<'window, 'client, 'conn> Drop for GlxContext<'window, 'client, 'conn>
{
	fn drop(&mut self)
	{
		unsafe { x11::glx::glXDestroyContext(self.client.xlib_display, self.context_ptr) };
	}
}

pub struct GlxWindowRenderer<'context, 'window, 'client, 'conn>
{
	pub glx_context: &'context GlxContext<'window, 'client, 'conn>,
}

impl<'context, 'window, 'client, 'conn> GlxWindowRenderer<'context, 'window, 'client, 'conn>
{
	pub fn new(glx_context: &'context GlxContext<'window, 'client, 'conn>)
	-> GlxWindowRenderer<'context, 'window, 'client, 'conn>
	{
		unsafe
		{
			x11::glx::glXMakeCurrent(
				glx_context.client.xlib_display,
				glx_context.window.id as x11::xlib::XID,
				glx_context.context_ptr
			)
		};

		return GlxWindowRenderer
		{
			glx_context,
		};
	}

	pub fn opengl_renderer(&self) -> crate::platform::opengl::OpenGlRenderer
	{
		return crate::platform::opengl::OpenGlRenderer::new(&self.glx_context.opengl_context);
	}
}

impl<'context, 'window, 'client, 'conn> Drop for GlxWindowRenderer<'context, 'window, 'client, 'conn>
{
	fn drop(&mut self)
	{
		unsafe
		{
			crate::platform::opengl::check_gl_error();
			x11::glx::glXSwapBuffers(self.glx_context.client.xlib_display, self.glx_context.window.id as x11::xlib::XID);
			x11::glx::glXMakeCurrent(self.glx_context.client.xlib_display, 0, std::ptr::null_mut());
		};
	}
}

pub struct Client<'conn>
{
	pub xlib_display: *mut x11::xlib::Display,
	pub client      : &'conn crate::platform::xcb::Client,
}

impl<'conn> Client<'conn>
{
	pub fn from_xcb_client(client: &'conn crate::platform::xcb::Client) -> Client<'conn>
	{
		return Client
		{
			xlib_display: client.conn.get_raw_dpy(),
			client,
		};
	}

	pub fn search_fb_configs<'client>(&'client self, screen_num: i32, visual_attribs: &[i32]) -> FbConfigsIterator<'client, 'conn>
	{
		let mut size: std::os::raw::c_int = 0;
		let root = unsafe
		{
			x11::glx::glXChooseFBConfig(
				self.xlib_display,
				screen_num,
				visual_attribs.as_ptr(),
				&mut size as *mut std::os::raw::c_int
			)
		};

		return FbConfigsIterator
		{
			size: size as isize,
			root,
			client: self,
			current_index : 0,
		};
	}

	pub fn sync(&self)
	{
		unsafe { x11::xlib::XSync(self.xlib_display, x11::xlib::False) };
	}

	//TODO(fpalacios): GLX_FLAG
	fn glx_extension(&self, window: &crate::platform::xcb::Window) -> String
	{
		let c_str_ptr = unsafe { x11::glx::glXQueryExtensionsString(self.xlib_display, window.screen.id)};

		return unsafe { std::ffi::CStr::from_ptr(c_str_ptr) }
			.to_str()
			.unwrap()
			.to_owned();
	}

	//TODO(fpalacios): GLX_FLAG
	pub fn check_glx_extension(&self, window: &crate::platform::xcb::Window, ext_name: &str) -> bool
	{
		let glx_exts = self.glx_extension(window);

		for glx_ext in glx_exts.split(" ")
		{
			if glx_ext == ext_name
			{
				return true;
			}
		}
		return false;
	}

	//TODO(fpalacios): GLX FLAG
	pub fn glx_version(dpy: *mut x11::xlib::Display) -> (i32, i32)
	{
		let mut major: std::os::raw::c_int = 0;
		let mut minor: std::os::raw::c_int = 0;
		unsafe
		{
			if x11::glx::glXQueryVersion(dpy, &mut major as *mut std::os::raw::c_int, &mut minor as *mut std::os::raw::c_int) == 0
			{
				panic!("cannot get glx version");
			}
		}
		return (major, minor);
	}

	//TODO(fpalacios): GLX_FLAG
	pub fn create_glx_context<'window, 'client, 'fb_configs>(
		&'client self,
		window    : &'window crate::platform::xcb::Window<'conn>,
		fb_configs: &FbConfigs
	)
	-> GlxContext<'window, 'client, 'conn>
	{
		if !self.check_glx_extension(window, "GLX_ARB_create_context")
		{
			panic!("could not find GLX extension GLX_ARB_create_context");
		}

		let glx_create_context_attribs: GlXCreateContextAttribsARBProc =
			unsafe { std::mem::transmute(load_gl_func("glXCreateContextAttribsARB")) };

		gl::load_with(|n| unsafe { load_gl_func(&n) });

		if !gl::GenVertexArrays::is_loaded()
		{
			panic!("no GL3 support available!");
		}

		if !gl::CreateShader::is_loaded()
		{
			panic!("No shader support");
		}

		if !gl::GenBuffers::is_loaded()
		{
			panic!("No buffer support");
		}

		unsafe { CTX_ERROR_OCCURRED = false };
		let old_handler = unsafe { x11::xlib::XSetErrorHandler(Some(ctx_error_handler)) };

		let context_attribs: [std::os::raw::c_int; 5] =
		[
			x11::glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB as std::os::raw::c_int, 3,
			x11::glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB as std::os::raw::c_int, 0,
			0
		];

		let ctx = unsafe
		{
			glx_create_context_attribs(
				self.xlib_display,
				fb_configs.glx_fb_config,
				std::ptr::null_mut(),
				x11::xlib::True,
				&context_attribs[0] as *const std::os::raw::c_int
			)
		};

		window.screen.client.flush().unwrap();
		self.sync();

		unsafe { x11::xlib::XSetErrorHandler(std::mem::transmute(old_handler)) };

		if ctx.is_null() || unsafe { CTX_ERROR_OCCURRED }
		{
			panic!("error when creating gl context");
		}

		if unsafe { x11::glx::glXIsDirect(self.xlib_display, ctx) } == 0
		{
			panic!("obtained indirect rendering context")
		}

		return GlxContext
		{
			window,
			client        : self,
			context_ptr   : ctx,
			opengl_context: crate::platform::opengl::OpenGlContext::new(),
		};
	}
}