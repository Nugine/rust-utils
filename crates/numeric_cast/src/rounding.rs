use core::any::type_name;
use core::fmt;

#[inline(always)]
#[must_use]
pub fn rounding_cast<X, Y>(x: X) -> Y
where
    Y: RoundingCastFrom<X>,
{
    Y::rounding_cast_from(x)
}

pub trait RoundingCast: Sized {
    #[inline(always)]
    #[must_use]
    fn rounding_cast<T: RoundingCastFrom<Self>>(self) -> T {
        T::rounding_cast_from(self)
    }
}

macro_rules! impl_rounding_cast {
($($t:ty,)*) => {
    $(
        impl RoundingCast for $t {}
    )*
};
}

impl_rounding_cast!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64,);

pub trait RoundingCastFrom<T>: Sized {
    fn rounding_cast_from(val: T) -> Self;
}

macro_rules! rounding_cast {
    (f32 => $rhs: ty) => {
        rounding_cast!(f32 => $rhs: float);
    };
    (f64 => $rhs: ty) => {
        rounding_cast!(f64 => $rhs: float);
    };
    ($lhs: ty => $rhs:ty) => {
        impl RoundingCastFrom<$lhs> for $rhs {
            #[inline(always)]
            #[must_use]
            fn rounding_cast_from(val: $lhs) -> $rhs {
                val as $rhs
            }
        }
    };
    ($lhs: ty => $rhs: ty: inf) => {
        impl RoundingCastFrom<$lhs> for $rhs {
            #[inline(always)]
            #[must_use]
            fn rounding_cast_from(val: $lhs) -> Self {
                let ans = val as $rhs;
                if ans.is_infinite() {
                    rounding_cast_failure::<$lhs, $rhs>(val)
                }
                ans
            }
        }
    };
    ($lhs: ty => $rhs: ty: float) => {
        impl RoundingCastFrom<$lhs> for $rhs {
            #[inline(always)]
            #[must_use]
            fn rounding_cast_from(val: $lhs) -> Self {
                if val.is_nan() || val.is_infinite() {
                    rounding_cast_failure::<$lhs, $rhs>(val)
                }
                val as $rhs
            }
        }
    }
}

rounding_cast!(u8    => f32);
rounding_cast!(u16   => f32);
rounding_cast!(u32   => f32);
rounding_cast!(u64   => f32);
rounding_cast!(u128  => f32: inf);
rounding_cast!(usize => f32);
rounding_cast!(i8    => f32);
rounding_cast!(i16   => f32);
rounding_cast!(i32   => f32);
rounding_cast!(i64   => f32);
rounding_cast!(i128  => f32);
rounding_cast!(isize => f32);

rounding_cast!(u8    => f64);
rounding_cast!(u16   => f64);
rounding_cast!(u32   => f64);
rounding_cast!(u64   => f64);
rounding_cast!(u128  => f64);
rounding_cast!(usize => f64);
rounding_cast!(i8    => f64);
rounding_cast!(i16   => f64);
rounding_cast!(i32   => f64);
rounding_cast!(i64   => f64);
rounding_cast!(i128  => f64);
rounding_cast!(isize => f64);

rounding_cast!(f32 => u8 );
rounding_cast!(f32 => u16);
rounding_cast!(f32 => u32);
rounding_cast!(f32 => u64);
rounding_cast!(f32 => u128);
rounding_cast!(f32 => usize);
rounding_cast!(f32 => i8 );
rounding_cast!(f32 => i16);
rounding_cast!(f32 => i32);
rounding_cast!(f32 => i64);
rounding_cast!(f32 => i128);
rounding_cast!(f32 => isize);

rounding_cast!(f64 => u8 );
rounding_cast!(f64 => u16);
rounding_cast!(f64 => u32);
rounding_cast!(f64 => u64);
rounding_cast!(f64 => u128);
rounding_cast!(f64 => usize);
rounding_cast!(f64 => i8 );
rounding_cast!(f64 => i16);
rounding_cast!(f64 => i32);
rounding_cast!(f64 => i64);
rounding_cast!(f64 => i128);
rounding_cast!(f64 => isize);

rounding_cast!(f32 => f64);

impl RoundingCastFrom<f64> for f32 {
    #[inline(always)]
    #[must_use]
    fn rounding_cast_from(val: f64) -> Self {
        let ans = val as f32;
        if val.is_nan() || val.is_infinite() || ans.is_infinite() {
            rounding_cast_failure::<f64, f32>(val)
        }
        ans
    }
}

#[cold]
#[track_caller]
#[inline(never)]
fn rounding_cast_failure<T, U>(val: T) -> !
where
    T: fmt::Display,
{
    crate::panic_failure("rounding_cast_failure", &val, type_name::<T>(), type_name::<U>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal() {
        let x: usize = usize::MAX / 2;
        let y: f64 = x.rounding_cast();
        assert_eq!(y, x as f64);

        let x: f64 = 1.0;
        let y: f32 = x.rounding_cast();
        assert_eq!(y, 1.0f32);

        let x: f32 = 300.0;
        let y: u8 = x.rounding_cast();
        assert_eq!(y, 255);
    }

    #[test]
    #[should_panic]
    fn int_overflow() {
        let x: u128 = u128::MAX;
        let _ = x.rounding_cast::<f32>();
    }

    #[test]
    #[should_panic]
    fn nan() {
        let x: f64 = f64::NAN;
        let _: f32 = rounding_cast(x);
    }

    #[test]
    #[should_panic]
    fn inf() {
        let x: f64 = f64::INFINITY;
        let _ = x.rounding_cast::<u8>();
    }
}
