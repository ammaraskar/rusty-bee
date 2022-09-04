use core::array::TryFromSliceError;

use self::security::SecurityHeader;
use byte::{BytesExt, LE};

#[derive(Debug, Clone)]
pub struct ParseError;
impl From<()> for ParseError {
    fn from(_: ()) -> Self {
        Self
    }
}
impl From<byte::Error> for ParseError {
    fn from(_: byte::Error) -> Self {
        Self
    }
}
impl From<TryFromSliceError> for ParseError {
    fn from(_: TryFromSliceError) -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct ZigbeePacket<'a> {
    pub frame_control_field: FrameControlField,
    pub destination: u16,
    pub source: u16,
    pub radius: u8,
    pub sequence_number: u8,
    pub extended_destination: Option<u64>,
    pub extended_source: Option<u64>,
    pub multicast_control: Option<u8>,
    pub security_header: Option<security::SecurityHeader>,
    pub payload: &'a [u8],
}
impl<'a> ZigbeePacket<'a> {
    pub fn try_parse_from(packet: &'a [u8]) -> Result<Self, ParseError> {
        let offset = &mut 0;

        let fcf = packet.read_with::<u16>(offset, LE)?;
        let fcf = FrameControlField::from(fcf);

        let destination = packet.read_with::<u16>(offset, LE)?;
        let source = packet.read_with::<u16>(offset, LE)?;

        let radius = packet.read_with::<u8>(offset, LE)?;
        let sequence_number = packet.read_with::<u8>(offset, LE)?;

        let extended_destination = match fcf.destination_present {
            true => Some(packet.read_with::<u64>(offset, LE)?),
            false => None,
        };
        let extended_source = match fcf.source_address_present {
            true => Some(packet.read_with::<u64>(offset, LE)?),
            false => None,
        };
        let multicast_control = match fcf.multicast_present {
            true => Some(packet.read_with::<u8>(offset, LE)?),
            false => None,
        };

        if fcf.source_route_present {
            let relay_count = packet.read_with::<u8>(offset, LE)?;
            let _relay_index = packet.read_with::<u8>(offset, LE)?;

            // Read the relay list which we'll ignore for now.
            for _ in 0..relay_count {
                let _relay = packet.read_with::<u8>(offset, LE)?;
            }
        }

        let security_header = match fcf.security_present {
            true => Some(SecurityHeader::try_parse_from(packet, offset)?),
            false => None,
        };

        // If there is a MAC, don't include it in the payload.
        let end_index = match security_header {
            Some(ref header) => packet.len() - header.message_integrity_code.len(),
            None => packet.len(),
        };

        let payload = &packet[*offset..end_index];

        Ok(ZigbeePacket {
            frame_control_field: fcf,
            destination,
            source,
            radius,
            sequence_number,
            extended_destination,
            extended_source,
            multicast_control,
            security_header,
            payload,
        })
    }
}

#[derive(Debug)]
pub struct FrameControlField {
    pub frame_type: FrameType,
    pub protocol_version: u8,
    pub discover_route: DiscoverRoute,
    pub multicast_present: bool,
    pub security_present: bool,
    pub source_route_present: bool,
    pub destination_present: bool,
    pub source_address_present: bool,
    pub end_device_initiator: bool,
}
impl From<u16> for FrameControlField {
    fn from(field: u16) -> Self {
        // Should never panic, all possible values covered by FrameType.
        let frame_type = FrameType::try_from((field & 0b11) as u8).unwrap();
        let protocol_version: u8 = ((field >> 2) & 0b1111) as u8;
        // Should never panic, all possible values covered by DiscoverRoute.
        let discover_route = DiscoverRoute::try_from(((field >> 6) & 0b11) as u8).unwrap();
        let multicast_present = ((field >> 8) & 1) == 1;
        let security_present = ((field >> 9) & 1) == 1;
        let source_route_present = ((field >> 10) & 1) == 1;
        let destination_present = ((field >> 11) & 1) == 1;
        let source_address_present = ((field >> 12) & 1) == 1;
        let end_device_initiator = ((field >> 13) & 1) == 1;

        Self {
            frame_type,
            protocol_version,
            discover_route,
            multicast_present,
            security_present,
            source_route_present,
            destination_present,
            source_address_present,
            end_device_initiator,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum DiscoverRoute {
    SurpressRouteDiscovery,
    EnableRouteDiscovery,
    Reserved,
}
impl TryFrom<u8> for DiscoverRoute {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(DiscoverRoute::SurpressRouteDiscovery),
            0b01 => Ok(DiscoverRoute::EnableRouteDiscovery),
            0b10 => Ok(DiscoverRoute::Reserved),
            0b11 => Ok(DiscoverRoute::Reserved),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum FrameType {
    Data,
    Command,
    Reserved,
    InterPAN,
}
impl TryFrom<u8> for FrameType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(FrameType::Data),
            0b01 => Ok(FrameType::Command),
            0b10 => Ok(FrameType::Reserved),
            0b11 => Ok(FrameType::InterPAN),
            _ => Err(()),
        }
    }
}

pub mod security {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub enum KeyIdentifier {
        Data,
        Network,
        KeyTransport,
        KeyLoad,
    }
    impl TryFrom<u8> for KeyIdentifier {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0b00 => Ok(KeyIdentifier::Data),
                0b01 => Ok(KeyIdentifier::Network),
                0b10 => Ok(KeyIdentifier::KeyTransport),
                0b11 => Ok(KeyIdentifier::KeyLoad),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum MessageIntegritySize {
        None,
        B32,
        B64,
        B128,
    }
    impl TryFrom<u8> for MessageIntegritySize {
        type Error = ();

        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0b00 => Ok(MessageIntegritySize::None),
                0b01 => Ok(MessageIntegritySize::B32),
                0b10 => Ok(MessageIntegritySize::B64),
                0b11 => Ok(MessageIntegritySize::B128),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct SecurityControlField {
        pub using_encryption: bool,
        pub message_integrity_size: MessageIntegritySize,
        pub key_identifier: KeyIdentifier,
        pub using_extended_nonce: bool,
    }
    impl From<u8> for SecurityControlField {
        fn from(field: u8) -> Self {
            //let using_encryption = (field & 1) == 1;
            //let message_integrity_size = MessageIntegritySize::try_from((field >> 1) & 0b11).unwrap();

            // TODO: apparently before Zigbee 2004 this is the default?
            // https://github.com/wireshark/wireshark/blob/69d54d6f8e668b6018375121ea2afb99f3dd0177/epan/dissectors/packet-zbee-security.c#L293-L298
            let using_encryption = true;
            let message_integrity_size = MessageIntegritySize::B32;

            let key_identifier = KeyIdentifier::try_from((field >> 3) & 0b11).unwrap();
            let using_extended_nonce = ((field >> 5) & 1) == 1;

            Self {
                using_encryption,
                message_integrity_size,
                key_identifier,
                using_extended_nonce,
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct SecurityHeader {
        pub security_control_field: SecurityControlField,
        pub frame_counter: u32,
        pub extended_source: Option<u64>,
        pub key_message_number: u8,
        pub message_integrity_code: [u8; 4],
    }
    impl SecurityHeader {
        pub fn try_parse_from(packet: &[u8], offset: &mut usize) -> Result<Self, ParseError> {
            let security_control_field = packet.read_with::<u8>(offset, LE)?;
            let security_control_field = SecurityControlField::from(security_control_field);

            let frame_counter = packet.read_with::<u32>(offset, LE)?;

            let extended_source = match security_control_field.using_extended_nonce {
                true => Some(packet.read_with::<u64>(offset, LE)?),
                false => None,
            };

            let key_message_number = packet.read_with::<u8>(offset, LE)?;

            // Last 4 bytes is the MAC?
            let message_integrity_code = packet[(packet.len() - 4)..packet.len()].try_into()?;

            Ok(Self {
                security_control_field,
                frame_counter,
                extended_source,
                key_message_number,
                message_integrity_code,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::security::*;
    use super::*;

    #[test]
    fn parses_data_frame_control_field_correctly() {
        let fcf = FrameControlField::from(0x0208);

        assert_eq!(fcf.frame_type, FrameType::Data);
        assert_eq!(fcf.protocol_version, 2);
        assert_eq!(fcf.discover_route, DiscoverRoute::SurpressRouteDiscovery);
        assert_eq!(fcf.multicast_present, false);
        assert_eq!(fcf.security_present, true);
        assert_eq!(fcf.source_route_present, false);
        assert_eq!(fcf.destination_present, false);
        assert_eq!(fcf.source_address_present, false);
        assert_eq!(fcf.end_device_initiator, false);
    }

    #[test]
    fn parses_command_frame_control_field_correctly() {
        let fcf = FrameControlField::from(0x1a09);

        assert_eq!(fcf.frame_type, FrameType::Command);
        assert_eq!(fcf.protocol_version, 2);
        assert_eq!(fcf.discover_route, DiscoverRoute::SurpressRouteDiscovery);
        assert_eq!(fcf.multicast_present, false);
        assert_eq!(fcf.security_present, true);
        assert_eq!(fcf.source_route_present, false);
        assert_eq!(fcf.destination_present, true);
        assert_eq!(fcf.source_address_present, true);
        assert_eq!(fcf.end_device_initiator, false);
    }

    #[test]
    fn parses_full_broadcast_packet() {
        let packet = b"\
\x09\x1a\xbc\x8d\x81\x01\x1d\x79\xe1\xc2\xd9\x01\x01\x88\x17\x00\
\x9e\xc0\x81\x08\x01\x88\x17\x00\x28\xe7\x08\x14\x01\x3b\xbd\x5d\
\x0b\x01\x88\x17\x00\x00\xdc\x0e\x9a\x26\x3f\x28\x6a\xf1\
";

        let packet = ZigbeePacket::try_parse_from(packet).unwrap();

        assert_eq!(packet.destination, 0x8dbc);
        assert_eq!(packet.source, 0x0181);
        assert_eq!(packet.radius, 29);
        assert_eq!(packet.sequence_number, 121);
        assert_eq!(packet.extended_destination, Some(0x00_17_88_01_01_d9_c2_e1));
        assert_eq!(packet.extended_source, Some(0x00_17_88_01_08_81_c0_9e));

        let security_header = packet.security_header.unwrap();
        assert_eq!(
            security_header.security_control_field.key_identifier,
            KeyIdentifier::Network
        );
        assert_eq!(security_header.frame_counter, 18090215);
        assert_eq!(security_header.key_message_number, 0);
        assert_eq!(
            security_header.message_integrity_code,
            [0x3f, 0x28, 0x6a, 0xf1]
        );

        assert_eq!(packet.payload, b"\xDC\x0E\x9A\x26");
    }

    #[test]
    fn parses_network_key_security_control_field() {
        let scf = SecurityControlField::try_from(0x28).unwrap();

        assert_eq!(scf.using_encryption, true);
        assert_eq!(scf.message_integrity_size, MessageIntegritySize::B32);
        assert_eq!(scf.key_identifier, KeyIdentifier::Network);
        assert_eq!(scf.using_extended_nonce, true);
    }
}
