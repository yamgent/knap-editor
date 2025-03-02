#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec2f {
    pub x: f64,
    pub y: f64,
}

impl Vec2f {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Bounds2f {
    pub pos: Vec2f,
    pub size: Vec2f,
}

impl Bounds2f {
    pub const ZERO: Self = Self {
        pos: Vec2f::ZERO,
        size: Vec2f::ZERO,
    };
}

pub trait ToU64 {
    fn to_u64(self) -> u64;
}

impl ToU64 for usize {
    fn to_u64(self) -> u64 {
        debug_assert!(std::mem::size_of::<usize>() == 8);

        #[allow(clippy::as_conversions)]
        let result = self as u64;

        result
    }
}

pub trait ToUsize {
    fn to_usize(self) -> usize;
}

impl ToUsize for u64 {
    fn to_usize(self) -> usize {
        debug_assert!(std::mem::size_of::<usize>() == 8);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::as_conversions)]
        let result = self as usize;

        result
    }
}

/// Allow conversion from a numeric type to another type,
/// with the loss of precision accepted.
///
/// Note that most of the time, this indicates that an API is
/// poorly designed. However, there could be situations where
/// the API design is beyond our control, and for those instances,
/// using `Lossy` is an acceptable alternative.
///
/// Most implementation of `Lossy` follows the behaviour as described
/// in [`as` documentation](https://doc.rust-lang.org/nightly/reference/expressions/operator-expr.html#semantics).
pub trait Lossy<U> {
    fn lossy(&self) -> U;
}

impl Lossy<f64> for usize {
    fn lossy(&self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::as_conversions)]
        let result = *self as f64;
        result
    }
}

impl Lossy<f64> for u64 {
    fn lossy(&self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::as_conversions)]
        let result = *self as f64;
        result
    }
}

impl Lossy<usize> for f64 {
    fn lossy(&self) -> usize {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::as_conversions)]
        let result = *self as usize;
        result
    }
}

impl Lossy<u16> for f64 {
    fn lossy(&self) -> u16 {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::as_conversions)]
        let result = *self as u16;
        result
    }
}

impl Lossy<u64> for f64 {
    fn lossy(&self) -> u64 {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::as_conversions)]
        let result = *self as u64;
        result
    }
}
