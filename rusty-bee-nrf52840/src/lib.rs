#![cfg_attr(not(test), no_std)]
use rusty_bee::ZigbeeHardware;
use rusty_bee::initialize_zigbee_stack;

pub struct NRF52840ZigbeeHardware {}

impl NRF52840ZigbeeHardware {
    pub fn new() -> Self {
        Self {  }
    }
}

impl ZigbeeHardware for NRF52840ZigbeeHardware {
    fn connect(&self) -> bool {
        true
    }
}

#[no_mangle]
pub extern "C" fn zigbee_init() -> i32 {
    let hardware = NRF52840ZigbeeHardware::new();
    if initialize_zigbee_stack(&hardware) {
        return 0;
    }
    return 1;
}

// Infinite loop panic handler.
#[cfg(not(test))]
mod panic_handler {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
