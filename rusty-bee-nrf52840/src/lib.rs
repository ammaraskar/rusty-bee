#![cfg_attr(not(test), no_std)]
use rusty_bee::initialize_zigbee_stack;
use rusty_bee::ZigbeeHardware;

mod ieee802154_radio;
use ieee802154_radio::IEEE802154Driver;
use rusty_bee::network_layer::ZigbeePacket;

pub mod factory_information;

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
pub extern "C" fn zigbee_init(num_reads: u32, param: u32) -> u64 {
    let hardware = NRF52840ZigbeeHardware::new();
    if !initialize_zigbee_stack(&hardware) {
        return 1;
    }

    let mut radio = IEEE802154Driver::new();

    if param == 1 {
        radio.broadcast_beacon();
    }

    for _ in 0..(num_reads / 2) {
        let packet = radio.read_packet();
        serial_println!("Packet: {:?}", packet);

        match packet {
            Ok((frame, _size)) => {
                let zigbee = ZigbeePacket::try_parse_from(frame.payload);
                serial_println!("Zigbee: {:?}", zigbee);
            }
            _ => {}
        }
    }

    if param == 1 {
        radio.send_assosciation_request();
    }
    for _ in 0..(num_reads / 2) {
        let packet = radio.read_packet();
        serial_println!("Packet: {:?}", packet);

        match packet {
            Ok((frame, _size)) => {
                let zigbee = ZigbeePacket::try_parse_from(frame.payload);
                serial_println!("Zigbee: {:?}", zigbee);
            }
            _ => {}
        }
    }

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
