macro_rules! c_type {
    ($vis:vis type $name:ident = fn($($params:tt)*);) => {
        $vis type $name = Option<extern "system" fn($($params)*)>;
    };
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
        #[derive(Clone, Copy, PartialEq, Eq)]
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
    ($vis:vis enum $name:ident;) => {
        #[repr(transparent)]
        $vis struct $name(pub u32);
    };
    ($vis:vis enum $name:ident { $($item:ident = $value:expr,)+ }) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis struct $name(pub u32);

        impl $name {
            $($vis const $item: Self = Self($value);)*
        }
    };
    (#[repr($ty:ty)] $vis:vis enum $name:ident { $($item:ident = $value:expr,)+ }) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis struct $name(pub $ty);

        impl $name {
            $($vis const $item: Self = Self($value);)*
        }
    };
    ($vis:vis union $name:ident $($rest:tt)*) => {
        #[repr(C)]
        $vis union $name $($rest)*
    };
}
