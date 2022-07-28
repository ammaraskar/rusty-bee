use super::radio_driver::RadioDriver;

pub struct IEEE802154Driver {
    pub radio_driver: RadioDriver,
    pub mac_address: u64,
}

impl IEEE802154Driver {
    pub fn new() -> Self {
        let device_id_register = FactoryInformationReader::new().device_id_registers;

        let mut mac_address: u64 = u64::from(device_id_register.device_id_high.read()) << 32;
        mac_address |= u64::from(device_id_register.device_id_low.read());

        let radio_driver = RadioDriver::new();

        return IEEE802154Driver { radio_driver, mac_address };
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
