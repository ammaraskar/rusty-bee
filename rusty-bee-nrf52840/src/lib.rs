#![cfg_attr(not(test), no_std)]
use rusty_bee::initialize_zigbee_stack;
use rusty_bee::ZigbeeHardware;

mod ieee802154_radio;
use ieee802154_radio::IEEE802154Driver;

#[macro_use]
pub mod debug_print;

pub struct NRF52840ZigbeeHardware {}

impl NRF52840ZigbeeHardware {
    pub fn new() -> Self {
        Self {}
    }
}

impl ZigbeeHardware for NRF52840ZigbeeHardware {
    fn connect(&self) -> bool {
        true
    }
}

#[no_mangle]
pub extern "C" fn zigbee_init() -> u64 {
    let hardware = NRF52840ZigbeeHardware::new();
    if !initialize_zigbee_stack(&hardware) {
        return 1;
    }

    let radio = IEEE802154Driver::new();

    serial_println!("Packet: {:?}", radio.read_packet());

    return radio.mac_address;
    //return 0;
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
