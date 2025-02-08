#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const GRAY: Self = Self {
        r: 128,
        g: 128,
        b: 128,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const DARK_RED: Self = Self { r: 128, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const DARK_GREEN: Self = Self { r: 0, g: 128, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const DARK_BLUE: Self = Self { r: 0, g: 0, b: 128 };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const DARK_YELLOW: Self = Self {
        r: 128,
        g: 128,
        b: 0,
    };
    pub const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const DARK_CYAN: Self = Self {
        r: 0,
        g: 128,
        b: 128,
    };
    pub const MAGENTA: Self = Self {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const DARK_MAGENTA: Self = Self {
        r: 128,
        g: 0,
        b: 128,
    };
}
