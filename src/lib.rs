#![no_std]

use cafe_rs::prelude::*;
use log::{Level, Metadata, Record, SetLoggerError};

use std::{cell::Cell, ffi::CString};

pub struct CafeLogger {
    level: Cell<Level>,
    console: Cell<bool>,
}

unsafe impl Sync for CafeLogger {}

impl CafeLogger {
    #[inline]
    const fn get() -> &'static Self {
        static LOGGER: CafeLogger = CafeLogger::default();
        &LOGGER
    }

    pub const fn default() -> Self {
        Self {
            level: Cell::new(Level::Warn),
            console: Cell::new(true),
        }
    }

    /// Log level
    pub fn level(self, level: Level) -> Self {
        Self::get().level.replace(level);
        self
    }

    /// Internal debug console
    pub fn console(self, enabled: bool) -> Self {
        Self::get().console.replace(enabled);
        self
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(Self::get().level.get().to_level_filter());
        log::set_logger(Self::get())
    }
}

impl log::Log for CafeLogger {
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
        }
    }

    fn flush(&self) {}
}

/// Convenience function for initializing the logger with default settings.
pub fn init() -> Result<(), SetLoggerError> {
    CafeLogger::default().init()
}
