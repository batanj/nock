use std::{error::Error, os::raw::c_void};

use khronos_egl::{self as egl, DynamicInstance, EGL1_5, OPENGL_ES_API};
use wayland_client::backend::ObjectId;
use wayland_egl::WlEglSurface;

pub struct Egl {
    pub instance: egl::DynamicInstance<egl::EGL1_5>,
    pub display: egl::Display,
    pub context: egl::Context,
    pub surface: egl::Surface,
    pub window: WlEglSurface,
}

impl Egl {
    pub fn bind(
        display_ptr: *mut c_void,
        surface_id: ObjectId,
        width: i32,
        height: i32,
    ) -> Result<Self, Box<dyn Error>> {
        if width <= 0 || height <= 0 {
            return Err("egl size".into());
        }

        let instance = unsafe { DynamicInstance::<EGL1_5>::load_required()? };
        let display = unsafe { instance.get_display(display_ptr as egl::NativeDisplayType) }
            .ok_or("eglGetDisplay failed")?;
        instance.initialize(display)?;
        instance.bind_api(OPENGL_ES_API)?;

        let contrib_attribs = [
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::NONE,
        ];
        let config = instance
            .choose_first_config(display, &contrib_attribs)?
            .ok_or("no config")?;
        let window = WlEglSurface::new(surface_id, width, height)?;
        let ctx_attribs = [egl::CONTEXT_MAJOR_VERSION, 2, egl::NONE];
        let context = instance.create_context(display, config, None, &ctx_attribs)?;
        let surface = unsafe {
            instance.create_window_surface(
                display,
                config,
                window.ptr() as _,
                Some(&[egl::NONE]),
            )?
        };
        instance.make_current(display, Some(surface), Some(surface), Some(context))?;

        Ok(Self {
            instance,
            display,
            context,
            surface,
            window,
        })
    }

    pub fn swap(&self) -> Result<(), khronos_egl::Error> {
        self.instance.swap_buffers(self.display, self.surface)?;
        Ok(())
    }

    pub fn clear_rgb(&self, r: f32, g: f32, b: f32) {
        self.instance
            .make_current(
                self.display,
                Some(self.surface),
                Some(self.surface),
                Some(self.context),
            )
            .ok();

        type ClearColor = unsafe extern "C" fn(f32, f32, f32, f32);
        type Clear = unsafe extern "C" fn(u32);
        const COLOR_BUFFER_BIT: u32 = 0x0000_4000;

        unsafe {
            let cc = self
                .instance
                .get_proc_address("glClearColor")
                .map(|p| std::mem::transmute::<_, ClearColor>(p));
            let cl = self
                .instance
                .get_proc_address("glClear")
                .map(|p| std::mem::transmute::<_, Clear>(p));
            if let (Some(cc), Some(cl)) = (cc, cl) {
                cc(r, g, b, 1.0);
                cl(COLOR_BUFFER_BIT);
            }
        }
    }
}
