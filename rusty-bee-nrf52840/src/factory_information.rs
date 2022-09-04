const FACTORY_INFORMATION_REGISTER_BASE: usize = 0x10000000;
const FACTORY_INFORMATION_DEVICE_ID_OFFSET: usize = 0x60;
#[repr(C)]
pub struct FactoryInformationRegisterDeviceId {
    device_id_high: volatile_register::RO<u32>,
    device_id_low: volatile_register::RO<u32>,
}

pub struct FactoryInformationReader {
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

    pub fn get_device_id(&self) -> u64 {
        let mut mac_address: u64 = u64::from(self.device_id_registers.device_id_high.read()) << 32;
        mac_address |= u64::from(self.device_id_registers.device_id_low.read());

        return mac_address;
    }
}
