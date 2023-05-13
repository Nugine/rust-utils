#[macro_export]
macro_rules! cfg_group {
    ($($item:item)*) => {
        $($item)*
    }
}

/// Calculates the offset of the specified field from the start of the named struct.
#[macro_export]
macro_rules! offset_of {
    ($ty: path, $field: tt) => {{
        use ::core::mem::MaybeUninit;
        use ::core::primitive::{u8, usize};
        use ::core::ptr;

        #[allow(
            unused_unsafe,
            clippy::as_conversions,
            clippy::unneeded_field_pattern,
            clippy::undocumented_unsafe_blocks
        )]
        const OFFSET: usize = unsafe {
            // ensure the type is a named struct
            // ensure the field exists and is accessible
            let $ty { $field: _, .. };

            // const since 1.36
            let uninit: MaybeUninit<$ty> = MaybeUninit::uninit();

            // const since 1.59
            // UnsafeCell needs feature(const_refs_to_cell)
            let base_ptr: *const $ty = uninit.as_ptr();

            // stable since 1.51
            let field_ptr: *const _ = ptr::addr_of!((*base_ptr).$field);

            // const since 1.38
            let base_addr = base_ptr.cast::<u8>();
            let field_addr = field_ptr.cast::<u8>();

            // const since 1.65
            field_addr.offset_from(base_addr) as usize
        };
        OFFSET
    }};
}
