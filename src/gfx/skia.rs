struct Skia {
    gr_context: skia_safe::gpu::DirectContext,
    surface: Option<skia_safe::Surface>,
}
