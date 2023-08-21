#[inline(always)]
#[must_use]
pub fn wrapping_cast<X>(x: X) -> X::Output
where
    X: WrappingCast,
{
    x.wrapping_cast()
}

pub trait WrappingCast: Sized {
    type Output;
    fn wrapping_cast(self) -> Self::Output;
}

macro_rules! wrapping_cast {
    ($lhs: ty=>$rhs:ty) => {
        impl WrappingCast for $lhs {
            type Output = $rhs;
            #[inline(always)]
            #[must_use]
            fn wrapping_cast(self) -> Self::Output {
                self as $rhs
            }
        }
    };
}

wrapping_cast!(u8    => i8   );
wrapping_cast!(u16   => i16  );
wrapping_cast!(u32   => i32  );
wrapping_cast!(u64   => i64  );
wrapping_cast!(u128  => i128 );
wrapping_cast!(usize => isize);
wrapping_cast!(i8    => u8   );
wrapping_cast!(i16   => u16  );
wrapping_cast!(i32   => u32  );
wrapping_cast!(i64   => u64  );
wrapping_cast!(i128  => u128 );
wrapping_cast!(isize => usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping() {
        let x: i8 = -1;

        let y: u8 = x.wrapping_cast();
        assert_eq!(y, 255);

        let y = wrapping_cast(usize::MAX / 2);
        assert_eq!(y, isize::MAX);

        let y: usize = wrapping_cast(isize::MAX);
        assert_eq!(y, usize::MAX / 2);
    }
}
