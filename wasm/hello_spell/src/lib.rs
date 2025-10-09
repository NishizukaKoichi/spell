#[no_mangle]
pub extern "C" fn hello() -> i32 {
    42
}

#[no_mangle]
pub extern "C" fn _start() {
    // Entry point for WASI
}
