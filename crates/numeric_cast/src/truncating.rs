#[inline(always)]
#[must_use]
pub fn truncating_cast<X, Y>(x: X) -> Y
where
    Y: TruncatingCastFrom<X>,
{
    Y::truncating_cast_from(x)
}

pub trait TruncatingCast: Sized {
    #[inline(always)]
    #[must_use]
    fn truncating_cast<T: TruncatingCastFrom<Self>>(self) -> T {
        T::truncating_cast_from(self)
    }
}

macro_rules! impl_truncating_cast {
($($t:ty,)*) => {
    $(
        impl TruncatingCast for $t {}
    )*
};
}

impl_truncating_cast!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize,);

pub trait TruncatingCastFrom<T>: Sized {
    fn truncating_cast_from(val: T) -> Self;
}

macro_rules! truncating_cast {
    ($lhs: ty=>$rhs:ty) => {
        impl TruncatingCastFrom<$lhs> for $rhs {
            #[inline(always)]
            #[must_use]
            fn truncating_cast_from(val: $lhs) -> $rhs {
                val as $rhs
            }
        }
    };
}

truncating_cast!(u16  => u8  );
truncating_cast!(u32  => u8  );
truncating_cast!(u64  => u8  );
truncating_cast!(u128 => u8  );
truncating_cast!(u32  => u16 );
truncating_cast!(u64  => u16 );
truncating_cast!(u128 => u16 );
truncating_cast!(u64  => u32 );
truncating_cast!(u128 => u32 );
truncating_cast!(u128 => u64 );

truncating_cast!(usize => u8    );
truncating_cast!(usize => u16   );
truncating_cast!(usize => u32   );
truncating_cast!(u32   => usize );
truncating_cast!(u64   => usize );
truncating_cast!(u128  => usize );

truncating_cast!(i16  => i8  );
truncating_cast!(i32  => i8  );
truncating_cast!(i64  => i8  );
truncating_cast!(i128 => i8  );
truncating_cast!(i32  => i16 );
truncating_cast!(i64  => i16 );
truncating_cast!(i128 => i16 );
truncating_cast!(i64  => i32 );
truncating_cast!(i128 => i32 );
truncating_cast!(i128 => i64 );

truncating_cast!(isize => i8    );
truncating_cast!(isize => i16   );
truncating_cast!(isize => i32   );
truncating_cast!(i32   => isize );
truncating_cast!(i64   => isize );
truncating_cast!(i128  => isize );

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncating() {
        let x: i16 = -1;
        let y: i8 = truncating_cast(x);
        assert_eq!(y, -1);

        let y: u8 = 256u32.truncating_cast();
        assert_eq!(y, 0);

        let y = 257u16.truncating_cast::<u8>();
        assert_eq!(y, 1);
    }
}
