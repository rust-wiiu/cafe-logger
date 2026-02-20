#![no_std]

use cafe_rs::prelude::*;
use log::{Level, Metadata, Record, SetLoggerError};

use cafe::net::socket::Socket;
use std::{
    cell::{Cell, RefCell},
    ffi::CString,
    net::{SocketAddrV4, ToSocketAddrs},
};

pub struct CafeLogger {
    level: Cell<Level>,
    console: Cell<bool>,
    udp: RefCell<Option<(Socket, SocketAddrV4)>>,
}

unsafe impl Sync for CafeLogger {}

impl CafeLogger {
    #[inline]
    const fn get() -> &'static Self {
        static LOGGER: CafeLogger = CafeLogger::default();
        &LOGGER
    }

    #[inline]
    pub const fn default() -> Self {
        Self {
            level: Cell::new(Level::Warn),
            console: Cell::new(true),
            udp: RefCell::new(None),
        }
    }

    /// Log level
    #[inline]
    pub fn level(self, level: Level) -> Self {
        Self::get().level.replace(level);
        self
    }

    /// Internal debug console
    #[inline]
    pub fn console(self, enabled: bool) -> Self {
        Self::get().console.replace(enabled);
        self
    }

    /// Network UDP port
    #[inline]
    pub fn udp(self, address: impl ToSocketAddrs) -> Self {
        Self::get().udp.replace(Some((
            Socket::udp().unwrap(),
            address.to_socket_addrs().unwrap().next().unwrap(),
        )));
        self
    }

    #[inline]
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(Self::get().level.get().to_level_filter());
        log::set_logger(Self::get())
    }
}

impl log::Log for CafeLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level.get()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let str = CString::new(format!("{} - {}\n", record.level(), record.args())).unwrap();
            if self.console.get() {
                unsafe {
                    sys::coreinit::debug::report(str.as_ptr());
                }
            }
            if let Some((socket, addr)) = self.udp.borrow().as_ref() {
                socket.send_to(str.as_bytes(), addr, None);
            }
        }
    }

    fn flush(&self) {}
}

/// Convenience function for initializing the logger with default settings.
pub fn init() -> Result<(), SetLoggerError> {
    CafeLogger::default().init()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        CafeLogger::default()
            .level(log::Level::Debug)
            .console(true)
            .init()
            .unwrap();
    }
}
