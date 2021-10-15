//!
//! This module contains different functions implemented as builtin in C
//!

use crate::traits::Integer;

#[cfg(target_pointer_width = "32")]
type UsizeTrue = u32;

#[cfg(target_pointer_width = "64")]
type UsizeTrue = u64;

/// Realization of C `sizeof`
#[inline(always)]
pub const fn sizeof <T> () -> UsizeTrue {
    core::mem::size_of::<T>() as UsizeTrue
}

/// Realization of C `x++` (a = after)
#[inline(always)]
pub fn inca <T: Integer> (x: &mut T) -> T {
    let copy = *x;
    x.add_one_u8(1);
    copy
}

/// Realization of C `++x` (b = before)
#[inline(always)]
pub fn incb <T: Integer> (x: &mut T) -> T {
    x.add_one_u8(1);
    *x
}

/// Realization of C `x--` (a = after)
#[inline(always)]
pub fn deca <T: Integer> (x: &mut T) -> T {
    let copy = *x;
    x.sub_one_u8(1);
    copy
}

/// Realization of C `--x` (b = before)
#[inline(always)]
pub fn decb <T: Integer> (x: &mut T) -> T {
    x.sub_one_u8(1);
    *x
}
