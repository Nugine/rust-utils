use crate::sealed::Internal;

pub trait OutputSize<A> {
    type Output;

    #[doc(hidden)]
    fn __internal(_: Internal) {}
}

macro_rules! impl_output_size {
    (($($ty:tt,)*)) => {
        impl<$($ty,)* F, O> OutputSize<($($ty,)*)> for F
        where
            F: FnOnce($($ty,)*) -> O ,
        {
            type Output = O;
        }
    };
}

impl_output_size!(());
impl_output_size!((A0,));
impl_output_size!((A0, A1,));
impl_output_size!((A0, A1, A2,));
impl_output_size!((A0, A1, A2, A3,));
impl_output_size!((A0, A1, A2, A3, A4,));
impl_output_size!((A0, A1, A2, A3, A4, A5,));
impl_output_size!((A0, A1, A2, A3, A4, A5, A6,));
impl_output_size!((A0, A1, A2, A3, A4, A5, A6, A7,));
impl_output_size!((A0, A1, A2, A3, A4, A5, A6, A7, A8,));
impl_output_size!((A0, A1, A2, A3, A4, A5, A6, A7, A8, A9,));
impl_output_size!((A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10,));
impl_output_size!((A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11,));

#[inline]
#[must_use]
pub const fn output_size<F, A>(_: &F) -> usize
where
    F: OutputSize<A>,
{
    core::mem::size_of::<F::Output>()
}
