use crate::factory_information::FactoryInformationReader;

use super::radio_driver::config_types::*;
use super::radio_driver::RadioDriver;

use byte::TryRead;
use byte::TryWrite;
use ieee802154::mac::Address;
use ieee802154::mac::AddressMode;
use ieee802154::mac::command::Command;
use ieee802154::mac::security::default::Unimplemented;
use ieee802154::mac::FooterMode;
use ieee802154::mac::Frame;
use ieee802154::mac::FrameContent;
use ieee802154::mac::FrameSerDesContext;

/// Class to hold the frame context to test outside of making a full driver.
pub struct IEEE802154SenderReceiverCtx<'a> {
    pub frame_context: FrameSerDesContext<'a, Unimplemented, Unimplemented>,
}

impl IEEE802154SenderReceiverCtx<'_> {
    fn generate_broadcast_beacon(&mut self, buffer: &mut [u8]) -> usize {
        let frame = Frame {
            header: ieee802154::mac::Header {
                frame_type: ieee802154::mac::FrameType::MacCommand,
                frame_pending: false,
                ack_request: false,
                pan_id_compress: false,
                seq_no_suppress: false,
                ie_present: false,
                version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                seq: 42,
                destination: Address::broadcast(&AddressMode::Short),
                source: None,
                auxiliary_security_header: None,
            },
            content: FrameContent::Command(Command::BeaconRequest),
            payload: &[],
            footer: [0, 0],
        };

        frame.try_write(buffer, &mut self.frame_context).unwrap_or(0)
    }
}

pub struct IEEE802154Driver<'a> {
    pub radio_driver: RadioDriver,
    pub mac_address: u64,
    pub sender_reciever_ctx: IEEE802154SenderReceiverCtx<'a>,
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
    radio.set_crc_configuration(2, true, 0x011021);

    // Set the frequency to the proper IEEE 802.15.4 channel.
    match channel {
        Channel::FIFTEEN => radio.set_frequency(2425).unwrap(),
    }
}

impl IEEE802154Driver<'_> {
    pub fn new() -> Self {
        let mac_address = FactoryInformationReader::new().get_device_id();

        let mut radio_driver = RadioDriver::new();
        // TODO: pass as argument
        configure_radio_driver(&mut radio_driver, Channel::FIFTEEN);

        let frame_context = FrameSerDesContext::new(FooterMode::None, None);

        return IEEE802154Driver {
            radio_driver,
            mac_address,
            sender_reciever_ctx: IEEE802154SenderReceiverCtx { frame_context }
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

    pub fn broadcast_beacon(&mut self) {
        let mut packet = [0u8; 128];
        let len = self.sender_reciever_ctx.generate_broadcast_beacon(&mut packet);
        self.radio_driver.write_packet_blocking(&packet[..len], (len + 2) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_beacon_request_packet() {
        let mut ctx = IEEE802154SenderReceiverCtx { frame_context: FrameSerDesContext::new(FooterMode::None, None) };

        let mut packet = [0u8; 128];
        let len = ctx.generate_broadcast_beacon(&mut packet);

        assert_eq!(&packet[..len], b"\x03\x08\x2a\xff\xff\xff\xff\x07");
    }
}
