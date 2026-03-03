//! A logger that prints [log][https://crates.io/crates/log] messages to configured output channels on the Wii U. It is the Rust alternative to the [WHBLog][https://github.com/devkitPro/wut/blob/master/libraries/libwhb/include/whb/log.h] functions in [wut][https://github.com/devkitPro/wut].
//!
//! # Example
//!
//! ```
//! use cafe_logger::CafeLogger;
//! fn main() {
//!     CafeLogger::new().init().unwrap();
//!
//!     log::warn!("This is an example message.");
//! }
//! ```

#![no_std]

use cafe_rs::prelude::*;

use log::{Level, Metadata, Record, SetLoggerError};
use std::{
    cell::{Cell, RefCell},
    net::{ToSocketAddrs, UdpSocket},
};

pub struct CafeLogger {
    level: Cell<Level>,
    console: Cell<bool>,
    udp: RefCell<Option<UdpSocket>>,
}

// This is only correct if the logger is once initialized from the main thread and never changed after that. Messages from threads may intertwine.
unsafe impl Sync for CafeLogger {}

impl CafeLogger {
    #[inline]
    const fn get() -> &'static Self {
        static LOGGER: CafeLogger = CafeLogger::new();
        &LOGGER
    }

    /// Initializes the global logger with default log level set to [Level::Warn] and [console] enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use cafe_logger::CafeLogger;
    /// CafeLogger::new().init().unwrap();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            level: Cell::new(Level::Warn),
            console: Cell::new(true),
            udp: RefCell::new(None),
        }
    }

    /// Sets the log level of the logger instance.
    ///
    /// # Example
    ///
    /// ```
    /// use cafe_logger::CafeLogger;
    /// use log::Level;
    /// CafeLogger::new().level(Level::Info).init().unwrap();
    /// ```
    #[inline]
    pub fn level(self, level: Level) -> Self {
        Self::get().level.replace(level);
        self
    }

    /// Enables or disables logging to the internal console.
    ///
    /// # Example
    ///
    /// ```
    /// use cafe_logger::CafeLogger;
    /// CafeLogger::new().console(false).init().unwrap();
    /// ```
    #[inline]
    pub fn console(self, enabled: bool) -> Self {
        Self::get().console.replace(enabled);
        self
    }

    /// Enables or disabled logging to a (remote) UDP port.
    ///
    /// # Example
    ///
    /// ```
    /// use cafe_logger::CafeLogger;
    /// CafeLogger::new().udp("1.2.3.4:4405").init().unwrap();
    /// ```
    #[inline]
    pub fn udp(self, address: impl ToSocketAddrs) -> Self {
        Self::get().udp.replace(Some({
            let udp = UdpSocket::bind("0.0.0.0:0").unwrap();
            udp.connect(address).unwrap();
            udp
        }));
        self
    }

    /// Initializes the global logger with the current settings of the logger instance. This method MUST be called ONCE in order for the logger to work.
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
            // Use heapless instead of CString to avoid allocations, which in turn enables the use of log regardless of allocator setup
            let mut str = heapless::String::<512>::new();
            use core::{fmt::Write, write};
            let _ = write!(str, "{} - {:.500}\n\0", record.level(), record.args());

            if self.console.get() {
                unsafe {
                    sys::coreinit::debug::report(str.as_ptr().cast());
                }
            }
            if let Some(udp) = self.udp.borrow().as_ref() {
                let _ = udp.send(str.as_bytes());
            }
        }
    }

    fn flush(&self) {}
}

/// Convenience function for initializing the logger with default settings.
///
/// Identical to calling `CafeLogger::new().init()`.
pub fn init() -> Result<(), SetLoggerError> {
    CafeLogger::new().init()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        CafeLogger::new()
            .level(log::Level::Debug)
            .console(true)
            .init()
            .unwrap();
    }
}
