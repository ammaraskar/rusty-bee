// Implemented externally with platform specific print functions.
#[cfg(not(test))]
extern "C" {
    fn write_string_from_rust(bytes: *const u8, len: usize);
}

pub struct ExternSerialWriter {}

impl ExternSerialWriter {
    pub fn write_fmt(&mut self, args: core::fmt::Arguments) -> core::fmt::Result {
        core::fmt::Write::write_fmt(self, args)
    }
}

#[cfg(not(test))]
impl core::fmt::Write for ExternSerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { write_string_from_rust(s.as_bytes().as_ptr(), s.as_bytes().len()) }
        Ok(())
    }
}

#[cfg(test)]
impl core::fmt::Write for ExternSerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        std::print!("{}", s);
        Ok(())
    }
}

// Debugging printing.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        #[allow(unused_must_use)]
        {
            let mut stm = $crate::debug_print::ExternSerialWriter{};
            stm.write_fmt(core::format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {
        #[allow(unused_must_use)]
        {
            let mut stm = $crate::debug_print::ExternSerialWriter{};
            stm.write_fmt(core::format_args!($($arg)*));
            stm.write_fmt(core::format_args!("\n"));
        }
    };
}
