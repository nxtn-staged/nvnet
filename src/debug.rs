// #[cfg(debug_assertions)]
macro_rules! debug_println {
    ($level:expr, $format:expr$(, $arg:expr)*$(,)?) => {
        unsafe {
            crate::windows::km::wdm::DbgPrintEx(
                crate::windows::shared::dpfilter::DPFLTR_TYPE::DPFLTR_IHVNETWORK_ID,
                $level,
                concat!("[VEth] ", $format, "\n\0").as_ptr(),
                $($arg),*
            )
        }
    };
}

// #[cfg(debug_assertions)]
macro_rules! debug_println_unsafe {
    ($level:expr, $format:expr$(, $arg:expr)*$(,)?) => {
        crate::windows::km::wdm::DbgPrintEx(
            crate::windows::shared::dpfilter::DPFLTR_TYPE::DPFLTR_IHVNETWORK_ID,
            $level,
            concat!("[VEth] ", $format, "\n\0").as_ptr(),
            $($arg),*
        )
    };
}

// #[cfg(debug_assertions)]
macro_rules! trace_println {
    ($format:expr$(, $arg:expr)*$(,)?) => {
        debug_println!(
            crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
            $format,
            $($arg),*
        )
    };
}

// #[cfg(debug_assertions)]
macro_rules! trace_entry {
    ($name:expr) => {
        debug_println!(
            crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
            concat!("--> ", $name),
        )
    };
}

// #[cfg(debug_assertions)]
macro_rules! trace_entry_args {
    ($name:expr, $fmt:expr$(, $arg:expr)*$(,)?) => {
        debug_println!(
            crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
            concat!("--> ", $name, " - ", $fmt),
            $($arg),*
        )
    };
}

// #[cfg(debug_assertions)]
macro_rules! trace_exit {
    ($name:expr) => {
        debug_println!(
            crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
            concat!("<-- ", $name),
        )
    };
}

// #[cfg(debug_assertions)]
macro_rules! trace_exit_status {
    ($name:expr, $status:expr) => {
        debug_println!(
            crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
            concat!("<-- ", $name, " - status: 0x%08x"),
            $status,
        )
    };
}

// #[cfg(debug_assertions)]
macro_rules! trace_exit_status_unsafe {
    ($name:expr, $status:expr) => {
        debug_println_unsafe!(
            crate::windows::shared::dpfilter::DPFLTR_TRACE_LEVEL,
            concat!("<-- ", $name, " - status: 0x%08x"),
            $status,
        )
    };
}
