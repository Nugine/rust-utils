#[inline(always)]
#[must_use]
pub fn extending_cast<X, Y>(x: X) -> Y
where
    Y: ExtendingCastFrom<X>,
{
    Y::extending_cast_from(x)
}

pub trait ExtendingCast: Sized {
    #[inline(always)]
    #[must_use]
    fn extending_cast<T: ExtendingCastFrom<Self>>(self) -> T {
        T::extending_cast_from(self)
    }
}

macro_rules! impl_extending_cast {
($($t:ty,)*) => {
    $(
        impl ExtendingCast for $t {}
    )*
};
}

impl_extending_cast!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize,);

pub trait ExtendingCastFrom<T>: Sized {
    fn extending_cast_from(val: T) -> Self;
}

macro_rules! extending_cast {
    ($lhs: ty=>$rhs:ty) => {
        impl ExtendingCastFrom<$lhs> for $rhs {
            #[inline(always)]
            #[must_use]
            fn extending_cast_from(val: $lhs) -> $rhs {
                val as $rhs
            }
        }
    };
}

extending_cast!(u8  => u16  );
extending_cast!(u8  => u32  );
extending_cast!(u8  => u64  );
extending_cast!(u8  => u128 );
extending_cast!(u16 => u32  );
extending_cast!(u16 => u64  );
extending_cast!(u16 => u128 );
extending_cast!(u32 => u64  );
extending_cast!(u32 => u128 );
extending_cast!(u64 => u128 );
extending_cast!(i8  => i16  );
extending_cast!(i8  => i32  );
extending_cast!(i8  => i64  );
extending_cast!(i8  => i128 );
extending_cast!(i16 => i32  );
extending_cast!(i16 => i64  );
extending_cast!(i16 => i128 );
extending_cast!(i32 => i64  );
extending_cast!(i32 => i128 );
extending_cast!(i64 => i128 );

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extending() {
        let x: i8 = -1;
        let y: i16 = extending_cast(x);
        assert_eq!(y, -1);

        let y: u32 = 255u8.extending_cast();
        assert_eq!(y, 255);

        let y = 255u8.extending_cast::<u32>();
        assert_eq!(y, 255);
    }
}
