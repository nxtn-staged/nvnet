use core::panic::PanicInfo;

#[panic_handler]
fn veth_panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        trace_println!(
            "panic at %s:%u:%u",
            location.file().as_ptr(),
            location.line(),
            location.column()
        );
    }
    loop {}
}
