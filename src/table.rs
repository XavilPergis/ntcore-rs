use entry::EntryMask;
use std::collections::HashMap;
use ::instance::Instance;
use ::entry::{Value, Entry, EntryType};
// use sys::{self};

// const PATH_SEPERATEOR: char = '/';

#[derive(Clone, Debug)]
pub struct NetworkTable<'c> {
    inst: &'c Instance,
    prefix: String,

    entry_cache: HashMap<String, Entry>,
}

impl<'c> NetworkTable<'c> {
    pub fn new(prefix: String, inst: &'c Instance) -> Self {
        NetworkTable { inst, prefix, entry_cache: HashMap::new() }
    }

    pub fn get_subtable(&self, key: &str) -> NetworkTable {
        NetworkTable {
            inst: self.inst,
            prefix: self.prefix.clone() + "/" + key,
            entry_cache: HashMap::new(),
        }
    }

    // NOTE: it's not required to return a mut ref because all the methods on `Entry` use a shared ptr.
    pub fn get(&mut self, name: &str) -> Entry {
        // destructure to avoid borrow checker issues; this allows us to use mut references to the
        // members of `self` at the same time as non-mut refs.
        let &mut NetworkTable { ref mut entry_cache, ref inst, ref prefix, .. } = self;
        *entry_cache.entry(prefix.clone() + name)
            .or_insert_with(|| inst.get_entry(&(prefix.clone() + "/" + name)))
    }

    pub fn set<V: Into<Value>>(&mut self, name: &str, value: V) -> Result<(), EntryType> {
        self.get(name).set(value)
    }

    pub fn put(&mut self, key: &str, val: Value) -> Result<(), EntryType> {
        self.get(key).set(val)
    }

    pub fn get_filtered(&mut self, prefix: &str, types: EntryMask) -> Vec<Entry> {
        let entries = self.inst.get_entries_filtered(&(self.prefix.clone() + prefix), types);

        // Cache all the entries.
        for entry in &entries {
            // Just don't cache entries that don't have UTF-8 names.
            if let Some(entry_name) = entry.name() {
                // Entry is a copy type, so we can just move out of the reference
                self.entry_cache.insert(entry_name, *entry);
            }
        }

        entries
    }
}
