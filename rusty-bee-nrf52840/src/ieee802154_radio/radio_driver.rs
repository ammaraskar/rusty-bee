use crate::serial_print;

pub struct RadioDriver {
    tasks: &'static mut RadioPeripheralTasks,
    events: &'static mut RadioPeripheralEvents,
    config_and_state: &'static mut RadioPeripheralConfigurationAndState,
}

impl RadioDriver {
    pub fn new() -> Self {
        let tasks = unsafe { &mut *(RADIO_TASKS_OFFSET as *mut RadioPeripheralTasks) };
        let events = unsafe { &mut *(RADIO_EVENTS_OFFST as *mut RadioPeripheralEvents) };
        let config_and_state = unsafe {
            &mut *(RADIO_CONFIG_AND_STATE_OFFSET as *mut RadioPeripheralConfigurationAndState)
        };

        serial_print!("Current radio state is: ");
        serial_print!("{:?}", config_and_state.radio_state.read());
        serial_print!("\n");

        return Self {
            tasks,
            events,
            config_and_state,
        };
    }
}

const RADIO_BASE_ADDRESS: usize = 0x40001000;

const RADIO_TASKS_OFFSET: usize = RADIO_BASE_ADDRESS + 0x0;
#[repr(C)]
struct RadioPeripheralTasks {
    /// TASK_TXEN in Nordic's datasheet.
    trigger_tx: volatile_register::WO<u32>,
    /// TASK_RXEN in Nordic's datasheet.
    trigger_rx: volatile_register::WO<u32>,
    /// TASKS_START in Nordic's datasheet.
    trigger_radio_start: volatile_register::WO<u32>,
    /// TASKS_STOP in Nordic's datasheet.
    trigger_radio_stop: volatile_register::WO<u32>,
    /// TASKS_DISABLE in Nordic's datasheet.
    trigger_radio_disable: volatile_register::WO<u32>,
    /// Starts RSSI measurement and takes one single sample of the received
    /// signal strength.
    ///
    /// TASKS_RSSISTART in Nordic's datasheet.
    trigger_rssi_start_measurement: volatile_register::WO<u32>,
    /// TASKS_RSSISTOP in Nordic's datasheet.
    trigger_rssi_stop: volatile_register::WO<u32>,
    /// TASKS_BCSTART in Nordic's datasheet.
    trigger_bit_counter_start: volatile_register::WO<u32>,
    /// TASKS_BCSTOP in Nordic's datasheet.
    trigger_bit_counter_stop: volatile_register::WO<u32>,
    /// TASKS_EDSTART in Nordic's datasheet.
    trigger_energy_detection_start: volatile_register::WO<u32>,
    /// TASKS_EDSTOP in Nordic's datasheet.
    trigger_energy_detection_stop: volatile_register::WO<u32>,
    /// TASKS_CCASTART in Nordic's datasheet.
    trigger_eclear_channel_assessment_start: volatile_register::WO<u32>,
    /// TASKS_CCASTOP in Nordic's datasheet.
    trigger_eclear_channel_assessment_stop: volatile_register::WO<u32>,
}

const RADIO_EVENTS_OFFST: usize = RADIO_BASE_ADDRESS + 0x100;
#[repr(C)]
struct RadioPeripheralEvents {
    /// 1 if RADIO has ramped up and is ready to be started.
    ///
    /// EVENTS_READY in Nordic's datasheet.
    events_ready: volatile_register::RW<u32>,
    /// 1 if RADIO has sent or received an address.
    ///
    /// EVENTS_ADDRESS in Nordic's datasheet.
    events_address: volatile_register::RW<u32>,
    /// 1 if RADIO sent or received a packet payload.
    ///
    /// EVENTS_PAYLOAD in Nordic's datasheet.
    events_payload: volatile_register::RW<u32>,
    /// 1 if RADIO is done sending or receiving a packet.
    ///
    /// EVENTS_END in Nordic's datasheet.
    events_packet_end: volatile_register::RW<u32>,
}

mod config_types {
    /// Values for the output power of the RADIO in decibel-milliwatts.
    #[derive(Copy, Clone)]
    #[repr(u32)]
    pub enum TransmissionPower {
        // +8 dBm
        Positive8dBm = 0x8,
        // +7 dBm
        Positive7dBm = 0x7,
        // +6 dBm
        Positive6dBm = 0x6,
        // +5 dBm
        Positive5dBm = 0x5,
        // +4 dBm
        Positive4dBm = 0x4,
        // +3 dBm
        Positive3dBm = 0x3,
        // +2 dBm
        Positive2dBm = 0x2,
        // 0 dBm
        ZerodBM = 0x0,
        // -4 dBm
        Negative4dBm = 0xFC,
        // -8 dBm
        Negative8dBm = 0xF8,
        // -12 dBm
        Negative12dBm = 0xF4,
        // -16 dBm
        Negative16dBm = 0xF0,
        // -20 dBm
        Negative20dBm = 0xEC,
        // -30 dBm
        Negative30dBm = 0xE2,
        // -40 dBm
        Negative40dBm = 0xD8,
    }

    /// Values for the data rate and modulation mode of the RADIO.
    #[derive(Copy, Clone)]
    #[repr(u32)]
    pub enum RadioMode {
        // 1 Mbit/s Nordic proprietary radio mode.
        Nrf1Mbit = 0,
        // 2 Mbit/s Nordic proprietary radio mode.
        Nrf2Mbit = 1,
        // 1 Mbit/s Bluetooth Low Energy.
        Ble1Mbit = 3,
        // 2 Mbit/s Bluetooth Low Energy.
        Ble2Mbit = 4,
        // Long Range Bluetooth Low Energy with 125kbit/s TX, 125 kbit/s and 500 kbit/s RX
        BleLR125kbit = 5,
        // Long Range Bluetooth Low Energy with 500kbit/s TX, 125 kbit/s and 500 kbit/s RX
        BleLR500kbit = 6,
        // IEEE 802.15.4-2006 at 250 kbit/s
        IEEE802154 = 15,
    }

    /// Values for the state of the RADIO.
    #[derive(Copy, Clone, Debug)]
    #[repr(u32)]
    pub enum RadioState {
        Disabled = 0,
        /// Radio is ramping up to receive data.
        RxRampUp = 1,
        /// Radio is ready to receive.
        RxIdle = 2,
        /// Reception has started and RXADDRESSES are being monitored.
        Rx = 3,
        /// Radio is disabling the receiver.
        RxDisable = 4,
        /// Radio is ramping up to transmit data.
        TxRampUp = 9,
        /// Radio is ready to transmit.
        TxIdle = 10,
        /// Radio is transmitting a packet.
        Tx = 11,
        /// Radio is disabling the transmitter.
        TxDisable = 12,
    }
}

const RADIO_CONFIG_AND_STATE_OFFSET: usize = RADIO_BASE_ADDRESS + 0x504;
#[repr(C)]
struct RadioPeripheralConfigurationAndState {
    /// Memory address of where the RADIO will use DMA to put the received
    /// packet or read the packet to transmit.
    ///
    /// PACKETPTR in Nordic's datasheet.
    packet_pointer: volatile_register::RW<usize>,
    /// Frequency selection:
    ///
    /// bits   8: Sets base frequency to 2400MHz if set to 0, else 2360MHz.
    /// bits 0-6: Value from 0 to 100 to add to base frequency.
    ///
    /// FREQUENCY in Nordic's datasheet.
    frequency: volatile_register::RW<u32>,
    /// TXPOWER in Nordic's datasheet.
    transmit_power: volatile_register::RW<config_types::TransmissionPower>,
    /// MODE in Nordic's datasheet.
    radio_mode: volatile_register::RW<config_types::RadioMode>,
    /// PCNF0 in Nordic's datasheet.
    packet_configuration_register_0: volatile_register::RW<u32>,
    /// PCNF1 in Nordic's datasheet.
    packet_configuration_register_1: volatile_register::RW<u32>,
    /// BASE0 in Nordic's datasheet.
    base_address_0: volatile_register::RW<u32>,
    /// BASE1 in Nordic's datasheet.
    base_address_1: volatile_register::RW<u32>,
    /// PREFIX0 in Nordic's datasheet.
    prefix_0: volatile_register::RW<u32>,
    /// PREFIX1 in Nordic's datasheet.
    prefix_1: volatile_register::RW<u32>,
    /// TXADDRESS in Nordic's datasheet.
    transmission_address: volatile_register::RW<u32>,
    /// RXADDRESSES in Nordic's datasheet.
    receive_addresses_selection: volatile_register::RW<u32>,
    /// CRCCNF in Nordic's datasheet.
    crc_configuration: volatile_register::RW<u32>,
    /// CRCPOLY in Nordic's datasheet.
    crc_polynomial: volatile_register::RW<u32>,
    /// CRCINIT in Nordic's datasheet.
    crc_initial_value: volatile_register::RW<u32>,
    /// TIFS in Nordic's datasheet.
    interface_spacing_microseconds: volatile_register::RW<u32>,
    /// RSSISAMPLE in Nordic's datasheet.
    rssi_sample: volatile_register::RO<u32>,
    /// STATE in Nordic's datasheet.
    radio_state: volatile_register::RO<config_types::RadioState>,
}
