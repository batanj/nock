use crate::domain::frame::Budget;

pub struct SurfaceId(pub u32);

pub struct Surface {
    pub id: SurfaceId,
    pub dirty: bool,
    pub animating: u32,
    // from frame.rs
    pub budget: Budget,
    // later: root: NodeId, size, scale
}
