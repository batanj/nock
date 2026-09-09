use crate::domain::surface::*;
use std::collections::HashMap;

pub struct App {
    pub surfaces: HashMap<SurfaceId, Surface>,
    // Phase 1+: conn, qh, egl, proto maps...
}
