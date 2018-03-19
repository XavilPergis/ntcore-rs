use sys::{self, NT_ConnectionInfo};
use std::net::IpAddr;
use ::NtString;

#[derive(Debug)]
pub struct ConnectionInfoEntry<'c>(&'c NT_ConnectionInfo);

impl<'c> ConnectionInfoEntry<'c> {
    // Strings live as long as the connection info.
    pub fn remote_id(&self) -> &str { unsafe { NtString(self.0.remote_id).as_str() } }
    pub fn remote_ip_str(&self) -> &str { unsafe { NtString(self.0.remote_ip).as_str() } }
    pub fn remote_ip(&self) -> IpAddr { self.remote_ip_str().parse().unwrap() }
    pub fn remote_port(&self) -> u32 { self.0.remote_port as u32 }
    pub fn last_update(&self) -> ::NetworkTime { ::NetworkTime(self.0.last_update) }
    pub fn protocol_version(&self) -> u32 { self.0.protocol_version as u32 }
}

impl<'c> PartialEq for ConnectionInfoEntry<'c> {
    fn eq(&self, other: &ConnectionInfoEntry) -> bool {
        self.last_update() == other.last_update() &&
        self.protocol_version() == other.protocol_version() &&
        self.remote_id() == other.remote_id() &&
        self.remote_ip() == other.remote_ip() &&
        self.remote_port() == other.remote_port()
    }
}

impl<'c> Eq for ConnectionInfoEntry<'c> {}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ConnectionInfo {
    ptr: *mut NT_ConnectionInfo,
    len: usize,
}

impl Drop for ConnectionInfo {
    fn drop(&mut self) {
        unsafe { sys::NT_DisposeConnectionInfoArray(self.ptr, self.len) }
    }
}

impl ConnectionInfo {
    pub(crate) fn from_raw(ptr: *mut NT_ConnectionInfo, len: usize) -> Self {
        ConnectionInfo { ptr, len }
    }

    #[inline]
    fn get(&self, idx: usize) -> Option<ConnectionInfoEntry> {
        unsafe {
            if idx >= self.len { None } else {
                Some(ConnectionInfoEntry(&*self.ptr.offset(idx as isize)))
            }
        }
    }
}

impl<'c> IntoIterator for &'c ConnectionInfo {
    type Item = ConnectionInfoEntry<'c>;
    type IntoIter = ConnectionInfoIter<'c>;

    fn into_iter(self) -> Self::IntoIter {
        ConnectionInfoIter(self, 0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ConnectionInfoIter<'c>(&'c ConnectionInfo, usize);

impl<'c> Iterator for ConnectionInfoIter<'c> {
    type Item = ConnectionInfoEntry<'c>;
    fn next(&mut self) -> Option<Self::Item> {
        self.1 += 1;
        self.0.get(self.1 - 1)
    }
}
