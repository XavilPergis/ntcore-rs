use table::NetworkTable;
use entry::EntryMask;
use std::os::raw::*;
use std::net::Ipv4Addr;
use std::ffi::CString;
use sys::{self, NT_Inst};
use ::connection::*;
use ::entry::Entry;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NetworkMode(u32);

impl NetworkMode {
    pub fn client(&self) -> bool { self.0 & sys::NT_NetworkMode_NT_NET_MODE_CLIENT != 0 }
    pub fn failure(&self) -> bool { self.0 & sys::NT_NetworkMode_NT_NET_MODE_FAILURE != 0 }
    pub fn none(&self) -> bool { self.0 & sys::NT_NetworkMode_NT_NET_MODE_NONE != 0 }
    pub fn server(&self) -> bool { self.0 & sys::NT_NetworkMode_NT_NET_MODE_SERVER != 0 }
    pub fn starting(&self) -> bool { self.0 & sys::NT_NetworkMode_NT_NET_MODE_STARTING != 0 }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Instance {
    handle: NT_Inst,
    server_mode: bool,
}

lazy_static! {
    static ref DEFAULT_INSTANCE: Instance = {
        Instance { handle: unsafe { sys::NT_GetDefaultInstance() }, server_mode: true }
    };
}

impl Instance {
    fn create_instance(server_mode: bool) -> Self {
        let handle = unsafe { sys::NT_CreateInstance() };
        Instance { handle, server_mode }
    }

    /// Get a reference to the default instance
    pub fn default_instance() -> &'static Instance {
        &*DEFAULT_INSTANCE
    }

    pub fn set_network_identity(&self, name: &str) {
        unsafe { sys::NT_SetNetworkIdentity(self.handle, name.as_ptr() as *const c_char, name.len()) }
    }
    // fn get_network_mode(&self) -> NetworkMode { unimplemented!() }

    pub fn start_server(persist_filename: String, listen_address: Ipv4Addr, port: u32) -> Instance {
        let inst = Instance::create_instance(true);
        let c_string = CString::new(persist_filename).unwrap(); // TODO
        // IP address should never have a null byte in the middle
        let addr_name = CString::new(listen_address.to_string()).unwrap();
        unsafe { sys::NT_StartServer(inst.handle, c_string.as_ptr(), addr_name.as_ptr(), port as c_uint) }
        inst
    }
    
    pub fn start_client_none() -> Instance {
        let inst = Instance::create_instance(false);
        unsafe { sys::NT_StartClientNone(inst.handle) }
        inst
    }
    
    pub fn start_client(server_ip: Ipv4Addr, port: u32) -> Instance {
        Instance::start_client_multi(vec![(&server_ip.to_string(), port)])
    }
    
    pub fn start_client_multi(servers: Vec<(&str, u32)>) -> Instance {
        let (mut ips, ports): (Vec<_>, Vec<_>) = servers.into_iter()
            .map(|(ip, port)| (ip.as_ptr() as *const c_char, port as c_uint))
            .unzip();
        let inst = Instance::create_instance(false);
        unsafe { sys::NT_StartClientMulti(inst.handle, ips.len(), ips.as_mut_slice().as_mut_ptr(), ports.as_slice().as_ptr()); }
        inst
    }
    
    pub fn start_client_team(team: u32, port: u32) -> Instance {
        let inst = Instance::create_instance(false);
        unsafe { sys::NT_StartClientTeam(inst.handle, team as c_uint, port as c_uint) }
        inst

    }

    pub fn set_server(&self, server_name: String, port: u32) {
        let c_string = CString::new(server_name).unwrap(); // TODO
        unsafe { sys::NT_SetServer(self.handle, c_string.as_ptr(), port as c_uint) }
    }

    // fn set_server_multi(&self, servers: Vec<(&str, u32)>) {
    //     unsafe { sys::NT_SetServerMulti }
    // }

    pub fn set_server_team(&self, team: u32, port: u32) {
        unsafe { sys::NT_SetServerTeam(self.handle, team as c_uint, port as c_uint) }
    }

    // fn start_DS_client(&self, port: u32) {}
    // fn stop_DS_client(&self) {}

    pub fn set_update_interval(&self, interval: f64) {
        unsafe { sys::NT_SetUpdateRate(self.handle, interval); }
    }

    /// Forces a flush of all entries to the network. This is usually done automatically, but this
    /// forces an immediate flush. However, to avoid network traffic, the flush may be delayed to
    /// some minimal interval between flushes.
    pub fn flush(&self) {
        unsafe { sys::NT_Flush(self.handle); }
    }

    /// Get all the connections on this instance. This will usually be one or zero on the client side
    /// but can be any number on the server side.
    pub fn get_connections(&self) -> ConnectionInfo {
        unsafe {
            let mut len = 0;
            let ptr = sys::NT_GetConnections(self.handle, &mut len);

            ConnectionInfo::from_raw(ptr, len)
        }
    }

    /// Get whether or not the instance is connected to another node
    pub fn is_connected(&self) -> bool {
        unsafe { sys::NT_IsConnected(self.handle) != 0 }
    }

    fn is_default_instance(&self) -> bool {
        unsafe { self.handle == sys::NT_GetDefaultInstance() }
    }

    // NT_Entry NT_GetEntry(NT_Inst inst, const char *name, size_t name_len);
    pub fn get_entry(&self, key: &str) -> Entry {
        let handle = unsafe { sys::NT_GetEntry(self.handle, key.as_ptr() as *const c_char, key.len()) };
        Entry::new(handle)
    }

    pub fn get_all_entries(&self) -> Vec<Entry> {
        // No prefix, don't care about the type.
        self.get_entries_filtered("", EntryMask::all())
    }

    pub fn get_entries_filtered(&self, prefix: &str, types: EntryMask) -> Vec<Entry> {
        // TODO: submit issue on wpilibsuite/ntcore; this currently causes UB on OOM
        unsafe {
            // Get entries from C
            let mut len = 0;
            let ptr = sys::NT_GetEntries(self.handle, prefix.as_ptr() as *const c_char,
                                         prefix.len(), types.0 as c_uint, &mut len);
            if ptr.is_null() && len != 0 { panic!("get_entries_filtered ran out of memory."); }
            let ret = ::std::slice::from_raw_parts(ptr, len).iter().map(|&handle| Entry { handle }).collect();
            // Free the C entry array; we've cloned it all.
            sys::NT_DisposeEntryArray(ptr, len);
            ret
        }
    }

    pub fn get_table(&self, name: String) -> NetworkTable {
        NetworkTable::new(name, self)
    }

    /// Delete ALL entries. Use with caution.
    pub fn delete_all_entries(&self) {
        unsafe { sys::NT_DeleteAllEntries(self.handle) }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if !self.is_default_instance() {
                if self.server_mode {
                    // We're a server.
                    sys::NT_StopClient(self.handle);
                } else {
                    // We're a client
                    sys::NT_StopClient(self.handle);
                }

                sys::NT_DestroyInstance(self.handle);
            }
        }
    }
}
