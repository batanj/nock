use crate::domain::SurfaceId;

pub struct ShellSurface {
    // link to domain
    pub id: SurfaceId,
    // Wayland - stays here
    // pub layer: LayerSurface,
    // egl window lives in gfx, referenced from runtime
}
