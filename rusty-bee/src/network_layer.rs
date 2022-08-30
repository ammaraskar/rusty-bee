#[derive(Debug, Clone)]
pub struct ParseError;
impl From<()> for ParseError {
    fn from(_: ()) -> Self {
        ParseError
    }
}

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
impl FrameControlField {
    pub fn parse_from(field: u16) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_data_frame_control_field_correctly() {
        let fcf = FrameControlField::parse_from(0x0208);

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
        let fcf = FrameControlField::parse_from(0x1a09);

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
}
