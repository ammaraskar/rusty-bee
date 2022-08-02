use crate::{serial_print, serial_println};

pub struct RadioDriver {
    tasks: &'static mut RadioPeripheralTasks,
    events: &'static mut RadioPeripheralEvents,
    shortcuts: &'static mut RadioPeripheralShortcuts,
    received_packet_details: &'static mut RadioPeripheralReceivedPacketDetails,
    config_and_state: &'static mut RadioPeripheralConfigurationAndState,
    power: &'static mut RadioPeripheralPower,
}

static mut PACKET_BUFFER: [u8; 512] = [0; 512];

impl RadioDriver {
    pub fn new() -> Self {
        let tasks = unsafe { &mut *(RADIO_TASKS_OFFSET as *mut RadioPeripheralTasks) };
        let events = unsafe { &mut *(RADIO_EVENTS_OFFST as *mut RadioPeripheralEvents) };
        let shortcuts = unsafe { &mut *(RADIO_SHORTCUTS_OFFSET as *mut RadioPeripheralShortcuts) };
        let received_packet_details = unsafe {
            &mut *(RADIO_RECEIVED_PACKET_DETAILS_OFFSET
                as *mut RadioPeripheralReceivedPacketDetails)
        };
        let config_and_state = unsafe {
            &mut *(RADIO_CONFIG_AND_STATE_OFFSET as *mut RadioPeripheralConfigurationAndState)
        };
        let power = unsafe { &mut *(RADIO_POWER_OFFSET as *mut RadioPeripheralPower) };

        serial_println!(
            "Current radio state is: {:?}",
            config_and_state.radio_state.read()
        );
        serial_println!(
            "Current radio mode is: {:?}",
            config_and_state.radio_mode.read()
        );

        // Set packetptr to point to our packet storage.
        unsafe {
            config_and_state
                .packet_pointer
                .write(PACKET_BUFFER.as_mut_ptr());
        }
        // Set max packet length to 255
        let mut configuration = config_types::PacketConfigurationRegister1::from_register(
            config_and_state.packet_configuration_register_1.read(),
        );
        configuration.max_packet_length = 255;
        unsafe {
            config_and_state
                .packet_configuration_register_1
                .write(configuration.pack_into_register());
        }

        return Self {
            tasks,
            events,
            shortcuts,
            received_packet_details,
            config_and_state,
            power,
        };
    }

    pub fn set_radio_mode(&mut self, mode: config_types::RadioMode) {
        unsafe {
            self.config_and_state.radio_mode.write(mode);
        }
    }

    pub fn set_packet_format(&mut self, config: config_types::PacketConfigurationRegister0) {
        unsafe {
            self.config_and_state
                .packet_configuration_register_0
                .write(config.pack_into_register());
        }
    }

    pub fn set_frequency(&mut self, frequency: u32) -> Result<(), ()> {
        if frequency < 2360 || frequency > 2500 {
            return Err(());
        }

        // The frequency register sets the frequency to either `2400 + x` or
        // `2360 + x` where `x` is the low 7 bits of the register.
        //
        // The base is switched by setting the 8th bit:
        // 0: 2400 + x
        // 1: 2360 + x
        let register: u32 = if frequency < 2400 {
            (frequency - 2360) | 0b100000000
        } else {
            frequency - 2400
        };

        unsafe {
            self.config_and_state.frequency.write(register);
        }

        return Ok(());
    }

    pub fn read_packet_blocking(&self) -> &[u8] {
        // Disable all shortcuts.
        unsafe {
            self.shortcuts.shortcuts.write(0);
        }
        // Trigger the RXEN task if needed.
        if self.config_and_state.radio_state.read() != config_types::RadioState::RxIdle {
            unsafe {
                self.tasks.trigger_rx_enable.write(1);
            }
        }

        serial_println!(
            "Current radio state is: {:?}",
            self.config_and_state.radio_state.read()
        );

        // Wait until we're in RxIdle state
        while self.config_and_state.radio_state.read() != config_types::RadioState::RxIdle {}
        serial_println!(
            "Current radio state is: {:?}",
            self.config_and_state.radio_state.read()
        );

        // Trigger the START task.
        unsafe {
            self.tasks.trigger_radio_start.write(1);
        }
        serial_println!("Start task triggered");
        for _ in 0..100 {
            serial_print!("{:?} ", self.config_and_state.radio_state.read());
        }
        serial_println!("");
        // Wait until we're back to RxIdle state
        while self.config_and_state.radio_state.read() != config_types::RadioState::RxIdle {}

        serial_println!("PAYLOAD event: {}", self.events.events_payload.read());
        unsafe {
            serial_println!("Packet length: {}", PACKET_BUFFER[0] as u32);
            serial_println!("{:X?}", &PACKET_BUFFER[0..20]);
        }
        serial_println!(
            "CRC matched: {:?}",
            self.received_packet_details.crc_status.read()
        );

        serial_println!("Might have finished reading??");
        // ======= Try to receive a packet??? =======
        unsafe {
            return &PACKET_BUFFER;
        }
    }
}

const RADIO_BASE_ADDRESS: usize = 0x40001000;

const RADIO_TASKS_OFFSET: usize = RADIO_BASE_ADDRESS + 0x0;
#[repr(C)]
struct RadioPeripheralTasks {
    /// TASK_TXEN in Nordic's datasheet.
    trigger_tx_enable: volatile_register::WO<u32>,
    /// TASK_RXEN in Nordic's datasheet.
    trigger_rx_enable: volatile_register::WO<u32>,
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

const RADIO_SHORTCUTS_OFFSET: usize = RADIO_BASE_ADDRESS + 0x200;
#[repr(C)]
struct RadioPeripheralShortcuts {
    /// SHORTS in Nordic's datasheet.
    shortcuts: volatile_register::RW<u32>,
}

const RADIO_RECEIVED_PACKET_DETAILS_OFFSET: usize = RADIO_BASE_ADDRESS + 0x400;
struct RadioPeripheralReceivedPacketDetails {
    /// CRCSTATUS in Nordic's datasheet.
    crc_status: volatile_register::RO<CrcStatus>,
    /// RXMATCH in Nordic's datasheet.
    received_logical_address: volatile_register::RO<u32>,
    /// RXCRC in Nordic's datasheet.
    received_crc: volatile_register::RO<u32>,
    /// DAI in Nordic's datasheet.
    received_address_match_index: volatile_register::RO<u32>,
    /// PDUSTAT in Nordic's datasheet.
    pdu_stat: volatile_register::RO<u32>,
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
enum CrcStatus {
    CrcError = 0,
    CrcOk = 1,
}

pub mod config_types {

    #[derive(Clone, Copy, Debug)]
    #[repr(u8)]
    pub enum PacketPreambleType {
        EightBit = 0,
        SixteenBit = 1,
        ThirtyTwoBit = 2,
        LongRange = 3,
    }

    /// Configures the format of the packet on the wire.
    ///
    /// The packet sent by the NRF52840 RADIO looks like:
    ///
    /// +----------+-------+--------+----+-------+----+--------+---+---------------+-----+------+
    /// | Preamble | Base  | Prefix | CI | Term1 | S0 | Length |S1 |    Payload    | CRC |Term2 |
    /// +----------+-------+--------+----+-------+----+--------+---+---------------+-----+------+
    ///                                                                                      
    ///            +----------------+                                                            
    ///                 Address   
    ///
    /// and these configuration options tweak the optional
    /// portions S0, Length, S1.
    #[derive(Debug)]
    pub struct PacketConfigurationRegister0 {
        /// LFLEN in Nordic's datasheet.
        pub length_field_num_bits: u8,
        /// S0LEN in Nordic's datasheet.
        pub s0_field_num_bytes: u8,
        /// S1LEN in Nordic's datasheet.
        pub s1_field_num_bits: u8,
        /// S1INCL in Nordic's datasheet.
        pub include_s1_field_in_buffer: bool,

        // Missing CILEN
        /// PLEN in Nordic's datasheet.
        pub preamble_length: PacketPreambleType,
        /// CRCINC in Nordic's datasheet.
        pub crc_included_in_length: bool,
        // Missing TERMLEN
    }
    impl PacketConfigurationRegister0 {
        pub fn new(
            length_field_num_bits: u8,
            s0_field_num_bytes: u8,
            s1_field_num_bits: u8,
            include_s1_field_in_buffer: bool,
            preamble_length: PacketPreambleType,
            crc_included_in_length: bool,
        ) -> Self {
            Self {
                length_field_num_bits,
                s0_field_num_bytes,
                s1_field_num_bits,
                include_s1_field_in_buffer,
                preamble_length,
                crc_included_in_length,
            }
        }

        pub fn pack_into_register(&self) -> u32 {
            let mut register: u32 = 0;
            register |= (self.length_field_num_bits as u32) & 0xF;
            register |= ((self.s0_field_num_bytes as u32) & 0x1) << 8;
            register |= ((self.s1_field_num_bits as u32) & 0xF) << 16;
            register |= ((self.include_s1_field_in_buffer as u32) & 0x1) << 20;
            register |= ((self.preamble_length as u32) & 0x3) << 24;
            register |= ((self.crc_included_in_length as u32) & 0x1) << 26;
            return register;
        }
    }

    #[derive(Debug)]
    pub struct PacketConfigurationRegister1 {
        /// Maximum length of the packet's payload, 0-255
        ///
        /// MAXLEN in Nordic's datasheet.
        pub max_packet_length: u8,
        /// The total length of the payload that the radio will pad to, even
        /// if the length is shorter, 0-255.
        ///
        /// STATLEN in Nordic's datasheet.
        pub padded_packet_length: u8,
        /// BALEN in Nordic's datasheet.
        pub base_address_length: u8,
        // Missing endianess and whitening for now.
    }
    impl PacketConfigurationRegister1 {
        pub fn new(
            max_packet_length: u8,
            padded_packet_length: u8,
            base_address_length: u8,
        ) -> Self {
            Self {
                max_packet_length,
                padded_packet_length,
                base_address_length,
            }
        }

        pub fn from_register(register: u32) -> Self {
            PacketConfigurationRegister1::new(
                (register & 0xFF) as u8,
                ((register >> 8) & 0xFF) as u8,
                ((register >> 16) & 0xFF) as u8,
            )
        }

        pub fn pack_into_register(&self) -> u32 {
            let mut register: u32 = 0;
            register |= (self.max_packet_length as u32) & 0xFF;
            register |= ((self.padded_packet_length as u32) & 0xFF) << 8;
            register |= ((self.base_address_length as u32) & 0xFF) << 16;
            return register;
        }
    }

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
    #[derive(Copy, Clone, Debug)]
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
    #[derive(Copy, Clone, Debug, PartialEq)]
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
    packet_pointer: volatile_register::RW<*mut u8>,
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

    // padding, CRCINIT is at 0x53C, TIFS is at 0x544
    pad_1: u32,

    /// TIFS in Nordic's datasheet.
    interface_spacing_microseconds: volatile_register::RW<u32>,
    /// RSSISAMPLE in Nordic's datasheet.
    rssi_sample: volatile_register::RO<u32>,

    // padding, RSSISAMPLE is at 0x548, STATE is at 0x550
    pad_2: u32,

    /// STATE in Nordic's datasheet.
    radio_state: volatile_register::RO<config_types::RadioState>,
}

const RADIO_POWER_OFFSET: usize = RADIO_BASE_ADDRESS + 0xFFC;
#[repr(C)]
struct RadioPeripheralPower {
    /// POWER in Nordic's datasheet.
    power: volatile_register::RW<u32>,
}
