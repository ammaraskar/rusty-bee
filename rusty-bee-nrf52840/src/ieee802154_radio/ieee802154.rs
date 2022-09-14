use crate::factory_information::FactoryInformationReader;
use crate::serial_println;

use super::radio_driver::config_types::*;
use super::radio_driver::RadioDriver;

use byte::TryRead;
use byte::TryWrite;
use ieee802154::mac::command::CapabilityInformation;
use ieee802154::mac::command::Command;
use ieee802154::mac::security::default::Unimplemented;
use ieee802154::mac::Address;
use ieee802154::mac::AddressMode;
use ieee802154::mac::ExtendedAddress;
use ieee802154::mac::FooterMode;
use ieee802154::mac::Frame;
use ieee802154::mac::FrameContent;
use ieee802154::mac::FrameSerDesContext;
use ieee802154::mac::FrameType;
use ieee802154::mac::PanId;
use ieee802154::mac::ShortAddress;

/// Class to hold the frame context to test outside of making a full driver.
pub struct IEEE802154SenderReceiverCtx<'a> {
    pub frame_context: FrameSerDesContext<'a, Unimplemented, Unimplemented>,
    pub short_address: Option<Address>,
    pub full_address: Option<Address>,
}
impl Default for IEEE802154SenderReceiverCtx<'_> {
    fn default() -> Self {
        Self {
            frame_context: FrameSerDesContext::new(FooterMode::None, None),
            short_address: None,
            full_address: None,
        }
    }
}

const FULL_ADDRESS: u64 = 0x42_42_42_42_42_42_42_42;
const PAN_ID: u16 = 0xd721;

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

        frame
            .try_write(buffer, &mut self.frame_context)
            .unwrap_or(0)
    }

    fn generate_assosciation_request(&mut self, buffer: &mut [u8]) -> usize {
        let frame = Frame {
            header: ieee802154::mac::Header {
                frame_type: ieee802154::mac::FrameType::MacCommand,
                frame_pending: false,
                ack_request: true,
                pan_id_compress: false,
                seq_no_suppress: false,
                ie_present: false,
                version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                seq: 43,
                destination: Some(Address::Short(PanId(PAN_ID), ShortAddress(0x8dbc))),
                source: Some(Address::Extended(
                    PanId::broadcast(),
                    ExtendedAddress(FULL_ADDRESS),
                )),
                auxiliary_security_header: None,
            },
            content: FrameContent::Command(Command::AssociationRequest(CapabilityInformation {
                full_function_device: true,
                mains_power: true,
                allocate_address: true,
                frame_protection: false,
                idle_receive: true,
            })),
            payload: &[],
            footer: [0, 0],
        };

        frame
            .try_write(buffer, &mut self.frame_context)
            .unwrap_or(0)
    }

    fn generate_data_request(&mut self, buffer: &mut [u8]) -> usize {
        let frame = Frame {
            header: ieee802154::mac::Header {
                frame_type: ieee802154::mac::FrameType::MacCommand,
                frame_pending: false,
                ack_request: true,
                pan_id_compress: true,
                seq_no_suppress: false,
                ie_present: false,
                version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                seq: 44,
                destination: Some(Address::Short(PanId(PAN_ID), ShortAddress(0x8dbc))),
                source: Some(Address::Extended(
                    PanId(PAN_ID),
                    ExtendedAddress(0x42_42_42_42_42_42_42_42),
                )),
                auxiliary_security_header: None,
            },
            content: FrameContent::Command(Command::DataRequest),
            payload: &[],
            footer: [0, 0],
        };

        frame
            .try_write(buffer, &mut self.frame_context)
            .unwrap_or(0)
    }

    fn generate_ack_packet(&mut self, sequence_number: u8, buffer: &mut [u8]) -> usize {
        let frame = Frame {
            header: ieee802154::mac::Header {
                frame_type: ieee802154::mac::FrameType::Acknowledgement,
                frame_pending: false,
                ack_request: false,
                pan_id_compress: false,
                seq_no_suppress: false,
                ie_present: false,
                version: ieee802154::mac::FrameVersion::Ieee802154_2003,
                seq: sequence_number,
                destination: None,
                source: None,
                auxiliary_security_header: None,
            },
            content: FrameContent::Acknowledgement,
            payload: &[],
            footer: [0, 0],
        };

        frame
            .try_write(buffer, &mut self.frame_context)
            .unwrap_or(0)
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

        return IEEE802154Driver {
            radio_driver,
            mac_address,
            sender_reciever_ctx: Default::default(),
        };
    }

    pub fn read_packet(&mut self) -> byte::Result<(Frame, usize)> {
        let packet_bytes = self.radio_driver.read_packet_blocking();

        let length: u8 = packet_bytes[0];
        // Slice the packet to the proper length.
        let packet_bytes = &packet_bytes[1..(length as usize)];

        // Read the frame from the packet bytes, no footer since we are using
        // the in-built CRC checking of the radio.
        let frame = Frame::try_read(packet_bytes, FooterMode::None);

        // Respond to the packet properly.
        if let Ok((packet, _)) = frame {
            if self.packet_intended_for_us(&packet) {
                self.maybe_ack_frame(&packet);

                self.react_to_packet(&packet);
            }
        }

        return frame;
    }

    fn packet_intended_for_us(&self, frame: &Frame) -> bool {
        if self.sender_reciever_ctx.full_address.is_some()
            && self.sender_reciever_ctx.full_address == frame.header.destination
        {
            return true;
        }
        if self.sender_reciever_ctx.short_address.is_some()
            && self.sender_reciever_ctx.short_address == frame.header.destination
        {
            return true;
        }
        return false;
    }

    /// Does not check if the packet is intended for us, check
    /// `self.packet_intended_for_us` before calling.
    fn maybe_ack_frame(&mut self, frame: &Frame) {
        if !frame.header.ack_request {
            return;
        }

        self.ack_frame(frame.header.seq);
    }

    fn ack_frame(&mut self, sequence_num: u8) {
        let mut packet = [0u8; 128];
        let len = self
            .sender_reciever_ctx
            .generate_ack_packet(sequence_num, &mut packet);
        self.radio_driver
            .write_packet_blocking(&packet[..len], (len + 2) as u8);
    }

    fn react_to_packet(&mut self, frame: &Frame) {
        serial_println!(
            "Got a packet intended for us! {:?}",
            frame.header.frame_type
        );
        match (frame.header.frame_type, frame.content) {
            (
                FrameType::MacCommand,
                FrameContent::Command(Command::AssociationResponse(address, status)),
            ) => {
                serial_println!("Assosciation status: {:?}", status);
                self.sender_reciever_ctx.short_address =
                    Some(Address::Short(PanId(PAN_ID), address));
            }
            _ => {}
        }
    }

    pub fn write_packet_and_expect_ack(
        &mut self,
        sequence_num: u8,
        packet: &[u8],
        length: u8,
    ) -> Result<(), ()> {
        self.radio_driver.write_packet_blocking(packet, length);

        for _ in 0..30 {
            // Make sure we got an ACK.
            match self.read_packet() {
                Ok((frame, _)) => {
                    if frame.header.frame_type == FrameType::Acknowledgement
                        && frame.header.seq == sequence_num
                    {
                        return Ok(());
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
        Err(())
    }

    pub fn broadcast_beacon(&mut self) {
        let mut packet = [0u8; 128];
        let len = self
            .sender_reciever_ctx
            .generate_broadcast_beacon(&mut packet);
        self.radio_driver
            .write_packet_blocking(&packet[..len], (len + 2) as u8);
    }

    pub fn send_assosciation_request(&mut self) {
        let mut packet = [0u8; 128];
        let len = self
            .sender_reciever_ctx
            .generate_assosciation_request(&mut packet);

        match self.write_packet_and_expect_ack(43, &packet[..len], (len + 2) as u8) {
            Ok(_) => {
                serial_println!("Assosciation successful!");
                self.sender_reciever_ctx.full_address = Some(Address::Extended(
                    PanId(PAN_ID),
                    ExtendedAddress(FULL_ADDRESS),
                ));
            }
            Err(_) => {
                serial_println!("Assosciation failed :(");
                return;
            }
        }

        // Send a data request!
        let len = self.sender_reciever_ctx.generate_data_request(&mut packet);
        match self.write_packet_and_expect_ack(44, &packet[..len], (len + 2) as u8) {
            Ok(_) => {
                serial_println!("Data request successful!")
            }
            Err(_) => {
                serial_println!("Data request failed :(");
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_beacon_request_packet() {
        let mut ctx: IEEE802154SenderReceiverCtx = Default::default();

        let mut packet = [0u8; 128];
        let len = ctx.generate_broadcast_beacon(&mut packet);

        assert_eq!(&packet[..len], b"\x03\x08\x2a\xff\xff\xff\xff\x07");
    }
}
