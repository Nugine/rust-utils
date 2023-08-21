use core::any::type_name;
use core::fmt;

#[inline]
#[must_use]
#[track_caller]
pub fn numeric_cast<X, Y>(x: X) -> Y
where
    Y: NumericCastFrom<X>,
{
    Y::numeric_cast_from(x)
}

pub trait NumericCast: Sized {
    #[inline]
    #[must_use]
    #[track_caller]
    fn numeric_cast<T: NumericCastFrom<Self>>(self) -> T {
        T::numeric_cast_from(self)
    }
}

macro_rules! impl_numeric_cast {
($($t:ty,)*) => {
    $(
        impl NumericCast for $t {}
    )*
};
}

impl_numeric_cast!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize,);

pub trait NumericCastFrom<T>: Sized {
    fn numeric_cast_from(val: T) -> Self;
}

macro_rules! cast {
($lhs: ty => $rhs: ty: nop) => {
    impl NumericCastFrom<$lhs> for $rhs {
        #[inline(always)]
        #[must_use]
        #[track_caller]
        fn numeric_cast_from(val: $lhs) -> $rhs {
            val
        }
    }
};

($lhs: ty => $rhs: ty: safe) => {
    impl NumericCastFrom<$lhs> for $rhs {
        #[inline(always)]
        #[must_use]
        #[track_caller]
        fn numeric_cast_from(val: $lhs) -> $rhs {
            val as _
        }
    }
};

($lhs: ty => $rhs: ty: overflow) => {
    impl NumericCastFrom<$lhs> for $rhs {
        #[inline]
        #[must_use]
        #[track_caller]
        fn numeric_cast_from(val: $lhs) -> Self {
            if val > <$rhs>::MAX as $lhs {
                numeric_cast_failure::<$lhs, $rhs>(val)
            }
            val as _
        }
    }
};

($lhs: ty => $rhs: ty: underflow) => {
    impl NumericCastFrom<$lhs> for $rhs {
        #[inline]
        #[must_use]
        #[track_caller]
        fn numeric_cast_from(val: $lhs) -> Self {
            if val < <$rhs>::MIN as $lhs {
                numeric_cast_failure::<$lhs, $rhs>(val)
            }
            val as _
        }
    }
};

($lhs: ty => $rhs: ty: both) => {
    impl NumericCastFrom<$lhs> for $rhs {
        #[inline]
        #[must_use]
        #[track_caller]
        fn numeric_cast_from(val: $lhs) -> Self {
            if val < <$rhs>::MIN as $lhs {
                numeric_cast_failure::<$lhs, $rhs>(val)
            }
            if val > <$rhs>::MAX as $lhs {
                numeric_cast_failure::<$lhs, $rhs>(val)
            }
            val as _
        }
    }
};

($lhs: ty => $rhs: ty: 16: $c16: tt, 32: $c32: tt, 64: $c64: tt) => {
    #[cfg(target_pointer_width = "16")]
    cast!($lhs => $rhs: $c16);

    #[cfg(target_pointer_width = "32")]
    cast!($lhs => $rhs: $c32);

    #[cfg(target_pointer_width = "64")]
    cast!($lhs => $rhs: $c64);
};
}

cast!(u8 => u8:     nop);
cast!(u8 => u16:    safe);
cast!(u8 => u32:    safe);
cast!(u8 => u64:    safe);
cast!(u8 => u128:   safe);
cast!(u8 => usize:  safe);
cast!(u8 => i8:     overflow);
cast!(u8 => i16:    safe);
cast!(u8 => i32:    safe);
cast!(u8 => i64:    safe);
cast!(u8 => i128:   safe);
cast!(u8 => isize:  safe);

cast!(u16 => u8:     overflow);
cast!(u16 => u16:    nop);
cast!(u16 => u32:    safe);
cast!(u16 => u64:    safe);
cast!(u16 => u128:   safe);
cast!(u16 => usize:  safe);
cast!(u16 => i8:     overflow);
cast!(u16 => i16:    overflow);
cast!(u16 => i32:    safe);
cast!(u16 => i64:    safe);
cast!(u16 => i128:   safe);
cast!(u16 => isize:  16: overflow, 32: safe, 64: safe);

cast!(u32 => u8:     overflow);
cast!(u32 => u16:    overflow);
cast!(u32 => u32:    nop);
cast!(u32 => u64:    safe);
cast!(u32 => u128:   safe);
cast!(u32 => usize:  16: overflow, 32: safe, 64: safe);
cast!(u32 => i8:     overflow);
cast!(u32 => i16:    overflow);
cast!(u32 => i32:    overflow);
cast!(u32 => i64:    safe);
cast!(u32 => i128:   safe);
cast!(u32 => isize:  16: overflow, 32: overflow, 64: safe);

cast!(u64 => u8:     overflow);
cast!(u64 => u16:    overflow);
cast!(u64 => u32:    overflow);
cast!(u64 => u64:    nop);
cast!(u64 => u128:   safe);
cast!(u64 => usize:  16: overflow, 32: overflow, 64: safe);
cast!(u64 => i8:     overflow);
cast!(u64 => i16:    overflow);
cast!(u64 => i32:    overflow);
cast!(u64 => i64:    overflow);
cast!(u64 => i128:   safe);
cast!(u64 => isize:  overflow);

cast!(u128 => u8:     overflow);
cast!(u128 => u16:    overflow);
cast!(u128 => u32:    overflow);
cast!(u128 => u64:    overflow);
cast!(u128 => u128:   nop);
cast!(u128 => usize:  overflow);
cast!(u128 => i8:     overflow);
cast!(u128 => i16:    overflow);
cast!(u128 => i32:    overflow);
cast!(u128 => i64:    overflow);
cast!(u128 => i128:   overflow);
cast!(u128 => isize:  overflow);

cast!(usize => u8:     overflow);
cast!(usize => u16:    16: safe, 32: overflow, 64: overflow);
cast!(usize => u32:    16: safe, 32: safe, 64: overflow);
cast!(usize => u64:    safe);
cast!(usize => u128:   safe);
cast!(usize => usize:  nop);
cast!(usize => i8:     overflow);
cast!(usize => i16:    overflow);
cast!(usize => i32:    16: safe, 32: overflow, 64: overflow);
cast!(usize => i64:    16: safe, 32: safe, 64: overflow);
cast!(usize => i128:   safe);
cast!(usize => isize:  overflow);

cast!(i8 => u8:     underflow);
cast!(i8 => u16:    underflow);
cast!(i8 => u32:    underflow);
cast!(i8 => u64:    underflow);
cast!(i8 => u128:   underflow);
cast!(i8 => usize:  underflow);
cast!(i8 => i8:     nop);
cast!(i8 => i16:    safe);
cast!(i8 => i32:    safe);
cast!(i8 => i64:    safe);
cast!(i8 => i128:   safe);
cast!(i8 => isize:  safe);

cast!(i16 => u8:     both);
cast!(i16 => u16:    underflow);
cast!(i16 => u32:    underflow);
cast!(i16 => u64:    underflow);
cast!(i16 => u128:   underflow);
cast!(i16 => usize:  underflow);
cast!(i16 => i8:     both);
cast!(i16 => i16:    nop);
cast!(i16 => i32:    safe);
cast!(i16 => i64:    safe);
cast!(i16 => i128:   safe);
cast!(i16 => isize:  safe);

cast!(i32 => u8:     both);
cast!(i32 => u16:    both);
cast!(i32 => u32:    underflow);
cast!(i32 => u64:    underflow);
cast!(i32 => u128:   underflow);
cast!(i32 => usize:  16: both, 32: underflow, 64: underflow);
cast!(i32 => i8:     both);
cast!(i32 => i16:    both);
cast!(i32 => i32:    nop);
cast!(i32 => i64:    safe);
cast!(i32 => i128:   safe);
cast!(i32 => isize:  16: both, 32: safe, 64: safe);

cast!(i64 => u8:     both);
cast!(i64 => u16:    both);
cast!(i64 => u32:    both);
cast!(i64 => u64:    underflow);
cast!(i64 => u128:   underflow);
cast!(i64 => usize:  16: both, 32: both, 64: underflow);
cast!(i64 => i8:     both);
cast!(i64 => i16:    both);
cast!(i64 => i32:    both);
cast!(i64 => i64:    nop);
cast!(i64 => i128:   safe);
cast!(i64 => isize:  16: both, 32: both, 64: safe);

cast!(i128 => u8:     both);
cast!(i128 => u16:    both);
cast!(i128 => u32:    both);
cast!(i128 => u64:    both);
cast!(i128 => u128:   underflow);
cast!(i128 => usize:  both);
cast!(i128 => i8:     both);
cast!(i128 => i16:    both);
cast!(i128 => i32:    both);
cast!(i128 => i64:    both);
cast!(i128 => i128:   nop);
cast!(i128 => isize:  both);

cast!(isize => u8:     both);
cast!(isize => u16:    16: underflow, 32: both, 64: both);
cast!(isize => u32:    16: underflow, 32: underflow, 64: both);
cast!(isize => u64:    underflow);
cast!(isize => u128:   underflow);
cast!(isize => usize:  underflow);
cast!(isize => i8:     both);
cast!(isize => i16:    16: safe, 32: both, 64: both);
cast!(isize => i32:    16: safe, 32: safe, 64: both);
cast!(isize => i64:    safe);
cast!(isize => i128:   safe);
cast!(isize => isize:  nop);

#[cold]
#[track_caller]
#[inline(never)]
fn numeric_cast_failure<T, U>(val: T) -> !
where
    T: fmt::Display,
{
    crate::panic_failure("numeric_cast_failure", &val, type_name::<T>(), type_name::<U>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nop() {
        let x: i8 = 127;
        let y = x.numeric_cast::<i8>();
        assert_eq!(y, 127);
    }

    #[test]
    fn extend() {
        let x: i8 = 127;
        let y = x.numeric_cast::<i16>();
        assert_eq!(y, 127);
    }

    #[test]
    fn truncate() {
        let x: i16 = 127;

        let y = x.numeric_cast::<i8>();
        assert_eq!(y, 127);

        let y: i8 = numeric_cast(x);
        assert_eq!(y, 127);
    }

    #[test]
    #[should_panic]
    fn overflow() {
        let x: i16 = 255;
        let _ = x.numeric_cast::<i8>();
    }

    #[test]
    #[should_panic]
    fn underflow() {
        let x: i16 = -1;
        let _: u8 = numeric_cast(x);
    }
}
