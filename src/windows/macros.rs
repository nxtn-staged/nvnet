macro_rules! c_type {
    ($vis:vis type $name:ident = fn($($params:tt)*) -> $ret:ty;) => {
        $vis type $name = Option<extern "system" fn($($params)*) -> $ret>;
    };
    ($vis:vis struct $name:ident;) => {
        #[repr(C)]
        $vis struct $name {
            _private: [u8; 0],
        }
    };
    ($(#[$attr:meta])* $vis:vis struct $name:ident($($rest:tt)*);) => {
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        $(#[$attr])*
        $vis struct $name($($rest)*);
    };
    ($(#[$attr:meta])* $vis:vis struct $name:ident $($rest:tt)*) => {
        #[repr(C)]
        $(#[$attr])*
        $vis struct $name $($rest)*

        impl Default for $name {
            fn default() -> Self {
                unsafe { core::mem::zeroed() }
            }
        }
    };
    ($vis:vis enum $($rest:tt)*) => {
        #[allow(dead_code)]
        #[repr(u32)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis enum $($rest)*
    };
    (#[repr($t:ident)] $vis:vis enum $($rest:tt)*) => {
        #[allow(dead_code)]
        #[repr($t)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis enum $($rest)*
    };
    (#[flags] $vis:vis enum $name:ident{$($item_name:ident = $item_expr:expr,)+}) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis struct $name(u32);

        #[allow(dead_code)]
        impl $name {
            $($vis const $item_name: $name = $name($item_expr);)+
        }

        impl core::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
    };
    ($vis:vis union $name:ident $($rest:tt)*) => {
        #[repr(C)]
        $vis union $name $($rest)*

        impl Default for $name {
            fn default() -> Self {
                unsafe { core::mem::zeroed() }
            }
        }
    };
}

macro_rules! declare_handle {
    ($name:ident) => {
        c_type!(
            pub struct $name(*mut core::ffi::c_void);
        );
    };
}
