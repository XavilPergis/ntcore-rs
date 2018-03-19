#![deny(missing_debug_implementations)]

extern crate ntcore_sys as sys;
#[macro_use]
extern crate lazy_static;

pub(crate) mod sealed {
    pub trait Sealed {}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NetworkTime(u64);

/// Helper for turning NT_String into things
#[derive(Debug)]
pub(crate) struct NtString(sys::NT_String);

impl NtString {
    // NT docs say NT_String is UTF-8, so unwrapping is fine
    unsafe fn as_str<'a>(self) -> &'a str { ::std::str::from_utf8(self.as_bytes()).unwrap() }
    unsafe fn as_bytes<'a>(self) -> &'a [u8] { ::std::slice::from_raw_parts(self.0.str as *mut u8, self.0.len) }
}

pub fn now() -> NetworkTime { unsafe { NetworkTime(sys::NT_Now()) } }

pub mod instance;
pub mod connection;
pub mod table;
pub mod entry;

pub use instance::Instance;
pub use table::NetworkTable;
