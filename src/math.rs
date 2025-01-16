#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Pos2u {
    pub x: u64,
    pub y: u64,
}

impl Pos2u {
    pub const ZERO: Self = Self { x: 0, y: 0 };
}
