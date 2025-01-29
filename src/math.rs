

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

impl AddCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: (i32, i32)) -> Self::Output {
        (
            self.0 + rhs.0,
            self.1 + rhs.1,
        )
    }
}

impl SubCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: (i32, i32)) -> Self::Output {
        (
            self.0 - rhs.0,
            self.1 - rhs.1,
        )
    }
}

impl MulCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: (i32, i32)) -> Self::Output {
        (
            self.0 * rhs.0,
            self.1 * rhs.1,
        )
    }
}

impl DivCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: (i32, i32)) -> Self::Output {
        (
            self.0 / rhs.0,
            self.1 / rhs.1,
        )
    }
}

impl RemCoord<(i32, i32)> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: (i32, i32)) -> Self::Output {
        (
            self.0 % rhs.0,
            self.1 % rhs.1,
        )
    }
}

// (i32, i32, i32)

impl AddCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
        )
    }
}

impl SubCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
        )
    }
}

impl MulCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (
            self.0 * rhs.0,
            self.1 * rhs.1,
            self.2 * rhs.2,
        )
    }
}

impl DivCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (
            self.0 / rhs.0,
            self.1 / rhs.1,
            self.2 / rhs.2,
        )
    }
}

impl RemCoord<(i32, i32, i32)> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: (i32, i32, i32)) -> Self::Output {
        (
            self.0 % rhs.0,
            self.1 % rhs.1,
            self.2 % rhs.2,
        )
    }
}

impl AddCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 + rhs,
            self.1 + rhs,
        )
    }
}

impl SubCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 - rhs,
            self.1 - rhs,
        )
    }
}

impl MulCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 * rhs,
            self.1 * rhs,
        )
    }
}

impl DivCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 / rhs,
            self.1 / rhs,
        )
    }
}

impl RemCoord<i32> for (i32, i32) {
    type Output = (i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 % rhs,
            self.1 % rhs,
        )
    }
}

impl AddCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn add_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 + rhs,
            self.1 + rhs,
            self.2 + rhs,
        )
    }
}

impl SubCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn sub_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 - rhs,
            self.1 - rhs,
            self.2 - rhs,
        )
    }
}

impl MulCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn mul_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 * rhs,
            self.1 * rhs,
            self.2 * rhs,
        )
    }
}

impl DivCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn div_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 / rhs,
            self.1 / rhs,
            self.2 / rhs,
        )
    }
}

impl RemCoord<i32> for (i32, i32, i32) {
    type Output = (i32, i32, i32);
    #[inline(always)]
    fn rem_coord(self, rhs: i32) -> Self::Output {
        (
            self.0 % rhs,
            self.1 % rhs,
            self.2 % rhs,
        )
    }
}

/// Since `i32::MAX - i32::MIN == u32::MAX`, it's possible to subtract
/// an i32 from an i32 where the result can fit into a u32 so long as the left-hand side is greater or equal
/// to the right-hand side.
pub const fn checked_sub_i32_to_u32(lhs: i32, rhs: i32) -> Option<u32> {
    if rhs > lhs {
        return None;
    }
    let lhs = lhs as i64;
    let rhs = rhs as i64;
    let result = lhs - rhs;
    Some(result as u32)
}

/// Since `i32::MAX - i32::MIN == u32::MAX`, it's possible to subtract
/// an i32 from an i32 where the result can fit into a u32 so long as the left-hand side is greater or equal
/// to the right-hand side.
/// 
/// In debug mode, this function will panic if `rhs > lhs`.
pub const fn sub_i32_to_u32(lhs: i32, rhs: i32) -> u32 {
    debug_assert!(rhs <= lhs);
    let lhs = lhs as i64;
    let rhs = rhs as i64;
    let result = lhs - rhs;
    result as u32
}

#[cfg(test)]
mod tests {
    use std::i32;

    use super::*;

    #[test]
    fn safe_math_test() {
        let result = checked_sub_i32_to_u32(i32::MAX, i32::MIN);
        assert_eq!(result, Some(u32::MAX));
        
    }

    #[test]
    fn coord_math_test() {
        let a = (1, 2);
        let b = (3, 4);
        assert_eq!(a.add_coord(b), (4, 6), "Add (i32, i32)");
        assert_eq!(a.sub_coord(b), (-2, -2), "Subtract (i32, i32)");
        assert_eq!(a.mul_coord(b), (3, 8), "Multiply (i32, i32)");
        assert_eq!(a.div_coord(b), (0, 0), "Divide (i32, i32)");
        assert_eq!(b.rem_coord(a), (0, 0), "Remainder (i32, i32)");
        let a = (33, 28, 14);
        let b = (5, 5, 5);
        assert_eq!(a.add_coord(b), (38, 33, 19), "Add (i32, i32, i32)");
        assert_eq!(a.sub_coord(b), (28, 23, 9), "Subtract (i32, i32, i32)");
        assert_eq!(a.mul_coord(b), (165, 140, 70), "Multiply (i32, i32, i32)");
        assert_eq!(a.div_coord(b), (6, 5, 2), "Divide (i32, i32, i32)");
        assert_eq!(a.rem_coord(b), (3, 3, 4), "Remainder (i32, i32, i32)");
        let a = (113, 144);
        let b = 5;
        assert_eq!(a.add_coord(b), (118, 149), "(i32, i32) + i32");
        assert_eq!(a.sub_coord(b), (108, 139), "(i32, i32) - i32");
        assert_eq!(a.mul_coord(b), (565, 720), "(i32, i32) * i32");
        assert_eq!(a.div_coord(b), (22, 28), "(i32, i32) / i32");
        assert_eq!(a.rem_coord(b), (3, 4), "(i32, i32) % i32");
        let a = (113, 144, 246);
        let b = 5;
        assert_eq!(a.add_coord(b), (118, 149, 251), "(i32, i32, i32) + i32");
        assert_eq!(a.sub_coord(b), (108, 139, 241), "(i32, i32, i32) - i32");
        assert_eq!(a.mul_coord(b), (565, 720, 1230), "(i32, i32, i32) * i32");
        assert_eq!(a.div_coord(b), (22, 28, 49), "(i32, i32, i32) / i32");
        assert_eq!(a.rem_coord(b), (3, 4, 1), "(i32, i32, i32) % i32");
    }
}