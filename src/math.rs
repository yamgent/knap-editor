#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Vec2u {
    pub x: u64,
    pub y: u64,
}

impl Vec2u {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub fn saturating_area(&self) -> u64 {
        self.x.saturating_mul(self.y)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Bounds2u {
    pub pos: Vec2u,
    pub size: Vec2u,
}

pub trait ToUsizeClamp
where
    Self: TryInto<usize>,
{
    /// This is used to get rid of the `clippy::cast_possible_truncation`
    /// lint error, as `Self` may have more bits than `usize`. When that happens,
    /// then we clamp the value to `usize::MAX`
    fn to_usize_clamp(self) -> usize {
        self.try_into().unwrap_or(usize::MAX)
    }
}

impl ToUsizeClamp for u64 {}

pub trait ToU16Clamp
where
    Self: TryInto<u16>,
{
    /// This is used to get rid of the `clippy::cast_possible_truncation`
    /// lint error, as `Self` may have more bits than `u16`. When that happens,
    /// then we clamp the value to `u16::MAX`
    fn to_u16_clamp(self) -> u16 {
        self.try_into().unwrap_or(u16::MAX)
    }
}

impl ToU16Clamp for u64 {}
impl ToU16Clamp for usize {}

pub trait ToU64 {
    fn to_u64(self) -> u64;
}

impl ToU64 for usize {
    fn to_u64(self) -> u64 {
        assert!(std::mem::size_of::<usize>() == 8);

        #[allow(clippy::as_conversions)]
        let result = self as u64;

        result
    }
}
