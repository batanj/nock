use crate::{
    domain::{layout::Measure, widget::HitTarget},
    theme::Theme,
};
use khronos_egl::{self as egl, DynamicInstance, EGL1_5, OPENGL_ES_API};
use skia_safe::{
    Color4f, ColorType, Font, FontMgr, FontStyle, Paint, Point, Surface,
    gpu::{
        self,
        gl::{Format::RGBA8, Interface},
    },
};
use std::{error::Error, os::raw::c_void};
use wayland_client::backend::ObjectId;
use wayland_egl::WlEglSurface;

pub struct Egl {
    pub instance: egl::DynamicInstance<egl::EGL1_5>,
    pub display: egl::Display,
    pub context: egl::Context,
    pub surface: egl::Surface,
    pub window: WlEglSurface,
    pub skia_surface: skia_safe::Surface,
    pub gr_context: skia_safe::gpu::DirectContext,
    pub font: Font,
}

impl Measure for Egl {
    fn text_width(&self, text: &str) -> u32 {
        self.font.measure_str(text, None).0.ceil() as u32
    }
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

        let interface = Interface::new_load_with(|name| {
            instance
                .get_proc_address(name)
                .map(|p| p as *const std::ffi::c_void)
                .unwrap_or(std::ptr::null())
        })
        .expect("interface");
        let mut gr_context = skia_safe::gpu::direct_contexts::make_gl(interface, None)
            .ok_or("DirectContext::new_gl")?;

        // wrap_skia
        let skia_surface = Egl::wrap_skia(&mut gr_context, width, height)?;

        let font_family = "JetBrainsMono Nerd Font";
        let font = Egl::init_font(font_family);

        Ok(Self {
            instance,
            display,
            context,
            surface,
            window,
            skia_surface,
            gr_context,
            font,
        })
    }

    fn wrap_skia(
        gr_context: &mut skia_safe::gpu::DirectContext,
        width: i32,
        height: i32,
    ) -> Result<Surface, Box<dyn Error>> {
        let fb_info = gpu::gl::FramebufferInfo {
            fboid: 0,
            format: RGBA8.into(),
            protected: gpu::Protected::No,
        };
        let backend_rt = gpu::backend_render_targets::make_gl((width, height), 0, 0, fb_info);

        let surface = gpu::surfaces::wrap_backend_render_target(
            gr_context,
            &backend_rt,
            gpu::SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .ok_or_else(|| String::from("wrap_backend_render_target returned None"))?;

        Ok(surface)
    }

    pub fn swap(&self) -> Result<(), khronos_egl::Error> {
        self.instance.swap_buffers(self.display, self.surface)?;
        Ok(())
    }

    pub fn draw_label(&mut self, hit_target: &HitTarget, theme: &Theme) {
        let surface = &mut self.skia_surface;
        let canvas = surface.canvas();

        let mut paint = Paint::new(Color4f::new(theme.fg.0, theme.fg.1, theme.fg.2, 1.0), None); // white
        paint.set_anti_alias(true);

        let x = hit_target.rect.x as f32 + hit_target.pad as f32;
        let y = hit_target.rect.y as f32 + hit_target.rect.height as f32 - 4.0;

        let origin = Point::new(x, y);

        canvas.draw_str(&hit_target.text, origin, self.font.as_ref(), &paint);
    }

    fn init_font(font_family: &str) -> Font {
        let font_mgr = FontMgr::new();
        let typeface = font_mgr
            .match_family_style(font_family, skia_safe::FontStyle::normal())
            .or_else(|| font_mgr.match_family_style("Noto Sans", FontStyle::normal()))
            .expect("error loading preferred font");
        let font_size = 14.0;
        Font::from_typeface(typeface, font_size)
    }

    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), Box<dyn Error>> {
        self.window.resize(width, height, 0, 0);
        self.instance
            .make_current(
                self.display,
                Some(self.surface),
                Some(self.surface),
                Some(self.context),
            )
            .ok();
        self.skia_surface = Egl::wrap_skia(&mut self.gr_context, width, height)?;

        Ok(())
    }

    pub fn begin_frame(&mut self, color: &Theme) {
        self.instance
            .make_current(
                self.display,
                Some(self.surface),
                Some(self.surface),
                Some(self.context),
            )
            .ok();
        let surface = &mut self.skia_surface;
        let canvas = surface.canvas();
        canvas.clear(skia_safe::Color4f::new(
            color.bg.0, color.bg.1, color.bg.2, 1.0,
        ));
    }

    pub fn end_frame(&mut self) {
        self.gr_context.flush_and_submit();
        self.swap().expect("egl swap");
    }
}
