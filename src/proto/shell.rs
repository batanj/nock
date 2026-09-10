use smithay_client_toolkit::shell::wlr_layer::LayerSurface;

use crate::domain::SurfaceId;

pub struct Shell {
    // link to domain
    pub id: SurfaceId,
    // Wayland - stays here
    // pub layer: LayerSurface,
    // egl window lives in gfx, referenced from runtime
    pub layer: LayerSurface,
}
