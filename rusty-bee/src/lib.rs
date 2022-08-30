#![cfg_attr(not(test), no_std)]

pub mod network_layer;

/// Initialize the Zigbee stack for specific hardware.
pub fn initialize_zigbee_stack<T: ZigbeeHardware>(hardware: &T) -> bool {
    hardware.connect()
}

/// All the hardware specific functions to implement for this library.
pub trait ZigbeeHardware {
    /// Connect and set up the radio hardware, returning true on success.
    fn connect(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use crate::ZigbeeHardware;

    pub struct TestHardware {}

    impl TestHardware {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl ZigbeeHardware for TestHardware {
        fn connect(&self) -> bool {
            false
        }
    }

    #[test]
    fn it_works() {
        let hardware = TestHardware::new();

        let result = super::initialize_zigbee_stack(&hardware);
        assert_eq!(result, false);
    }
}
