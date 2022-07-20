#![cfg_attr(not(test), no_std)]

// Infinite loop panic handler.
#[cfg(not(test))]
mod panic_handler {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
}


#[no_mangle]
pub extern "C" fn rust_function(a: i32, b: i32) -> i32 {
    return a + b;
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = super::rust_function(40, 2);
        assert_eq!(result, 42);
    }
}
