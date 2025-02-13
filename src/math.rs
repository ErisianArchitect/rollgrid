pub struct TupleConverter;

pub trait DimensionsType: Sized {}

pub trait ConvertTuple<A: DimensionsType, B: DimensionsType> {
    fn convert(input: A) -> B;
}

pub trait Convert: DimensionsType {
    fn convert<T>(self) -> T
    where
        T: DimensionsType,
        TupleConverter: ConvertTuple<Self, T>;
}

pub trait AddCoord<Rhs> {
    type Output;
    fn add_coord(self, rhs: Rhs) -> Self::Output;
}

pub trait SubCoord<Rhs> {
    type Output;
    fn sub_coord(self, rhs: Rhs) -> Self::Output;
}

pub trait MulCoord<Rhs> {
    type Output;
    fn mul_coord(self, rhs: Rhs) -> Self::Output;
}

pub trait DivCoord<Rhs> {
    type Output;
    fn div_coord(self, rhs: Rhs) -> Self::Output;
}

pub trait RemCoord<Rhs> {
    type Output;
    fn rem_coord(self, rhs: Rhs) -> Self::Output;
}

// ******** IMPLEMENTATIONS ********

impl DimensionsType for (i32, i32) {}
impl DimensionsType for (i32, i32, i32) {}
impl DimensionsType for (u32, u32) {}
impl DimensionsType for (u32, u32, u32) {}
impl DimensionsType for (i64, i64) {}
impl DimensionsType for (i64, i64, i64) {}
impl DimensionsType for (usize, usize) {}
impl DimensionsType for (usize, usize, usize) {}

impl ConvertTuple<(i32, i32), (i64, i64)> for TupleConverter {
    fn convert(input: (i32, i32)) -> (i64, i64) {
        (input.0 as i64, input.1 as i64)
    }
}

impl ConvertTuple<(u32, u32), (i64, i64)> for TupleConverter {
    fn convert(input: (u32, u32)) -> (i64, i64) {
        (input.0 as i64, input.1 as i64)
    }
}

impl ConvertTuple<(u32, u32), (usize, usize)> for TupleConverter {
    fn convert(input: (u32, u32)) -> (usize, usize) {
        (input.0 as usize, input.1 as usize)
    }
}

impl ConvertTuple<(i32, i32, i32), (i64, i64, i64)> for TupleConverter {
    fn convert(input: (i32, i32, i32)) -> (i64, i64, i64) {
        (input.0 as i64, input.1 as i64, input.2 as i64)
    }
}

impl ConvertTuple<(u32, u32, u32), (i64, i64, i64)> for TupleConverter {
    fn convert(input: (u32, u32, u32)) -> (i64, i64, i64) {
        (input.0 as i64, input.1 as i64, input.2 as i64)
    }
}

impl ConvertTuple<(u32, u32, u32), (usize, usize, usize)> for TupleConverter {
    fn convert(input: (u32, u32, u32)) -> (usize, usize, usize) {
        (input.0 as usize, input.1 as usize, input.2 as usize)
    }
}

impl<A: DimensionsType> Convert for A {
    fn convert<T>(self) -> T
    where
        T: DimensionsType,
        TupleConverter: ConvertTuple<Self, T>,
    {
        TupleConverter::convert(self)
    }
}

impl AddCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: (i32, i32)) -> Self::Output {
        (self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl SubCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: (i32, i32)) -> Self::Output {
        (self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl MulCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: (i32, i32)) -> Self::Output {
        (self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl DivCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: (i32, i32)) -> Self::Output {
        (self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl RemCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: (i32, i32)) -> Self::Output {
        (self.0 % rhs.0, self.1 % rhs.1)
    }
}

// (i32, i32, i32)

impl AddCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl SubCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl MulCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl DivCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (self.0 / rhs.0, self.1 / rhs.1, self.2 / rhs.2)
    }
}

impl RemCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (self.0 % rhs.0, self.1 % rhs.1, self.2 % rhs.2)
    }
}

impl AddCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: i32) -> Self::Output {
        (self.0 + rhs, self.1 + rhs)
    }
}

impl SubCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: i32) -> Self::Output {
        (self.0 - rhs, self.1 - rhs)
    }
}

impl MulCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: i32) -> Self::Output {
        (self.0 * rhs, self.1 * rhs)
    }
}

impl DivCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: i32) -> Self::Output {
        (self.0 / rhs, self.1 / rhs)
    }
}

impl RemCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: i32) -> Self::Output {
        (self.0 % rhs, self.1 % rhs)
    }
}

impl AddCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: i32) -> Self::Output {
        (self.0 + rhs, self.1 + rhs, self.2 + rhs)
    }
}

impl SubCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: i32) -> Self::Output {
        (self.0 - rhs, self.1 - rhs, self.2 - rhs)
    }
}

impl MulCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: i32) -> Self::Output {
        (self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl DivCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: i32) -> Self::Output {
        (self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl RemCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: i32) -> Self::Output {
        (self.0 % rhs, self.1 % rhs, self.2 % rhs)
    }
}

#[inline]
pub(crate) const fn add_u32_to_i32(i32_value: i32, u32_value: u32) -> i32 {
    let conv = i32_to_u32(i32_value);
    debug_assert!(u32::MAX - conv >= u32_value);
    u32_to_i32(conv + u32_value)
}

#[inline]
pub(crate) const fn checked_add_u32_to_i32(i32_value: i32, u32_value: u32) -> Option<i32> {
    let conv = i32_to_u32(i32_value);
    if let Some(result) = conv.checked_add(u32_value) {
        Some(u32_to_i32(result))
    } else {
        None
    }
}

#[inline]
pub(crate) const fn i32_to_u32(i32_value: i32) -> u32 {
    (i32_value as u32) ^ 0x8000_0000
}

#[inline]
pub(crate) const fn u32_to_i32(u32_value: u32) -> i32 {
    (u32_value ^ 0x8000_0000) as i32
}
