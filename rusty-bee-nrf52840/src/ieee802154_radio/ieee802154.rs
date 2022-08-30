use super::radio_driver::config_types::*;
use super::radio_driver::RadioDriver;

use byte::TryRead;
use ieee802154::mac::FooterMode;
use ieee802154::mac::Frame;

pub struct IEEE802154Driver {
    pub radio_driver: RadioDriver,
    pub mac_address: u64,
}

pub enum Channel {
    FIFTEEN,
}

pub fn configure_radio_driver(radio: &mut RadioDriver, channel: Channel) {
    // Set mode to IEEE
    radio.set_radio_mode(RadioMode::IEEE802154);
    // Configure the packet format.
    let packet_config =
        PacketConfigurationRegister0::new(8, 0, 0, false, PacketPreambleType::ThirtyTwoBit, true);
    radio.set_packet_format(packet_config);

    // Set the frequency to the proper IEEE 802.15.4 channel.
    match channel {
        Channel::FIFTEEN => radio.set_frequency(2425).unwrap(),
    }
}

impl IEEE802154Driver {
    pub fn new() -> Self {
        let device_id_register = FactoryInformationReader::new().device_id_registers;

        let mut mac_address: u64 = u64::from(device_id_register.device_id_high.read()) << 32;
        mac_address |= u64::from(device_id_register.device_id_low.read());

        let mut radio_driver = RadioDriver::new();
        // TODO: pass as argument
        configure_radio_driver(&mut radio_driver, Channel::FIFTEEN);

        return IEEE802154Driver {
            radio_driver,
            mac_address,
        };
    }

    pub fn read_packet(&self) -> byte::Result<(Frame, usize)> {
        let packet_bytes = self.radio_driver.read_packet_blocking();

        let length: u8 = packet_bytes[0];
        // Slice the packet to the proper length.
        let packet_bytes = &packet_bytes[1..(length as usize)];

        // Read the frame from the packet bytes, no footer since we are using
        // the in-built CRC checking of the radio.
        Frame::try_read(packet_bytes, FooterMode::None)
    }
}

const FACTORY_INFORMATION_REGISTER_BASE: usize = 0x10000000;
const FACTORY_INFORMATION_DEVICE_ID_OFFSET: usize = 0x60;
#[repr(C)]
struct FactoryInformationRegisterDeviceId {
    device_id_high: volatile_register::RO<u32>,
    device_id_low: volatile_register::RO<u32>,
}

struct FactoryInformationReader {
    device_id_registers: &'static mut FactoryInformationRegisterDeviceId,
}

impl FactoryInformationReader {
    pub fn new() -> Self {
        const ADDRESS: usize =
            FACTORY_INFORMATION_REGISTER_BASE + FACTORY_INFORMATION_DEVICE_ID_OFFSET;
        let device_id_registers =
            unsafe { &mut *(ADDRESS as *mut FactoryInformationRegisterDeviceId) };
        return FactoryInformationReader {
            device_id_registers: device_id_registers,
        };
    }
}
