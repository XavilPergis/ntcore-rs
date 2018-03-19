use std::os::raw::c_char;
use sys::{self, NT_Entry, NT_Value, NT_Type, NT_Bool, NT_String};
use ::{NetworkTime, NtString};

type ValueUnionBoolArr = sys::NT_Value__bindgen_ty_1__bindgen_ty_1;
type ValueUnionDoubleArr = sys::NT_Value__bindgen_ty_1__bindgen_ty_2;
type ValueUnionStringArr = sys::NT_Value__bindgen_ty_1__bindgen_ty_3;

/// Owned value from a network table entry. The data is cloned from the table.
#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    BoolArray(Vec<bool>),
    Double(f64),
    DoubleArray(Vec<f64>),
    String(String),
    StringArray(Vec<String>),
    Raw(Vec<u8>),
}

macro_rules! map_value {
    ($name:ident, $value_name:ident: $type:ty) => {
        pub fn $name<F: Fn($type) -> $type>(self, func: F) -> Value {
            if let Value::$value_name(inner) = self {
                Value::$value_name(func(inner))
            } else {
                self
            }
        }
    }
}

impl Value {
    fn to_nt_value(self, last_change: u64) -> NT_Value {
        macro_rules! value {($ty:ident, $last_change:expr, $what:ident: $what2:expr) => {
            NT_Value {
                type_: sys::$ty, last_change: $last_change,
                data: sys::NT_Value__bindgen_ty_1 { $what: $what2 }
            }
        }}

        fn to_nt_string<S: AsRef<[u8]>>(val: S) -> NT_String {
            NT_String { len: val.as_ref().len(), str: val.as_ref().as_ptr() as *mut c_char }
        }

        fn bool_array(val: Vec<bool>, last_change: u64) -> NT_Value {
            // C code has a different representation for bools, we need to allocate here :(
            let mut c_arr = val.into_iter().map(|val| val as i32).collect::<Vec<_>>();
            value!(NT_Type_NT_BOOLEAN_ARRAY, last_change, arr_boolean: ValueUnionBoolArr {
                size: c_arr.len(), arr: c_arr.as_mut_ptr()
            })
        }

        fn string_array(val: Vec<String>, last_change: u64) -> NT_Value {
            let mut c_arr = val.into_iter().map(to_nt_string).collect::<Vec<_>>();
            value!(NT_Type_NT_STRING_ARRAY, last_change, arr_string: ValueUnionStringArr {
                size: c_arr.len(), arr: c_arr.as_mut_ptr()
            })
        }

        fn double_array(mut val: Vec<f64>, last_change: u64) -> NT_Value {
            let arr_double = ValueUnionDoubleArr { size: val.len(), arr: val.as_mut_ptr() };
            value!(NT_Type_NT_DOUBLE_ARRAY, last_change, arr_double: arr_double)
        }

        match self {
            Value::Bool(val) => value!(NT_Type_NT_BOOLEAN, last_change, v_boolean: val as NT_Bool),
            Value::BoolArray(val) => bool_array(val, last_change),
            Value::Double(val) => value!(NT_Type_NT_DOUBLE, last_change, v_double: val),
            Value::DoubleArray(val) => double_array(val, last_change),
            Value::String(val) => value!(NT_Type_NT_STRING, last_change, v_string: to_nt_string(val)),
            Value::StringArray(val) => string_array(val, last_change),
            Value::Raw(val) => value!(NT_Type_NT_RAW, last_change, v_raw: to_nt_string(val))
        }
    }

    map_value!(map_bool, Bool: bool);
    map_value!(map_double, Double: f64);
    map_value!(map_string, String: String);
    map_value!(map_bool_array, BoolArray: Vec<bool>);
    map_value!(map_double_array, DoubleArray: Vec<f64>);
    map_value!(map_string_array, StringArray: Vec<String>);
    map_value!(map_raw, Raw: Vec<u8>);
}

macro_rules! impl_from {
    ($name:ident: $type:ty) => {
        impl From<$type> for Value {
            fn from(val: $type) -> Self { Value::$name(val) }
        }
    }
}

impl_from!(Bool: bool);
impl_from!(Double: f64);
impl_from!(String: String);
impl_from!(BoolArray: Vec<bool>);
impl_from!(DoubleArray: Vec<f64>);
impl_from!(StringArray: Vec<String>);
impl_from!(Raw: Vec<u8>);

/// Struct used for filtering entry types. You can create the mask like so:
/// ```rs
/// let mask = EntryMask::new(EntryType::Boolean) | EntryType::Double | EntryType::String;
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EntryMask(pub(crate) u32);

impl EntryMask {
    pub fn new(ty: EntryType) -> Self { EntryMask(ty as u32) }
    /// When getting entries, 0 explicitly means you don't care about any of the types of entries
    /// returned. `EntryMask::all() | EntryType::Boolean` is the same as `EntryMask::new(EntryType::Boolean)`
    pub fn all() -> Self { EntryMask(0) }
}

impl ::std::ops::BitOr<EntryType> for EntryMask {
    type Output = EntryMask;
    fn bitor(self, rhs: EntryType) -> EntryMask {
        EntryMask(self.0 | rhs as u32)
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EntryType {
    Boolean = sys::NT_Type_NT_BOOLEAN,
    BooleanArray = sys::NT_Type_NT_BOOLEAN_ARRAY,
    Double = sys::NT_Type_NT_DOUBLE,
    DoubleArray = sys::NT_Type_NT_DOUBLE_ARRAY,
    Raw = sys::NT_Type_NT_RAW,
    Rpc = sys::NT_Type_NT_RPC,
    String = sys::NT_Type_NT_STRING,
    StringArray = sys::NT_Type_NT_STRING_ARRAY,
    /// Entry does not exist
    Unassigned = sys::NT_Type_NT_UNASSIGNED,
}

impl From<NT_Type> for EntryType {
    /// Panics if the type is not one of the values of EntryType
    fn from(ty: NT_Type) -> Self {
        match ty {
            sys::NT_Type_NT_BOOLEAN => EntryType::Boolean,
            sys::NT_Type_NT_BOOLEAN_ARRAY => EntryType::BooleanArray,
            sys::NT_Type_NT_DOUBLE => EntryType::Double,
            sys::NT_Type_NT_DOUBLE_ARRAY => EntryType::DoubleArray,
            sys::NT_Type_NT_RAW => EntryType::Raw,
            sys::NT_Type_NT_RPC => EntryType::Rpc,
            sys::NT_Type_NT_STRING => EntryType::String,
            sys::NT_Type_NT_STRING_ARRAY => EntryType::StringArray,
            sys::NT_Type_NT_UNASSIGNED => EntryType::Unassigned,
            ty => panic!("Invalid NT_Type: {}", ty),
        }
    }
}

/// A handle to a possibly existant network table entry. 
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Entry {
    pub(crate) handle: NT_Entry,
}

impl Entry {
    pub(crate) fn new(handle: NT_Entry) -> Self { Entry { handle } }

    pub fn entry_type(&self) -> EntryType {
        unsafe { sys::NT_GetEntryType(self.handle).into() }
    }

    pub fn exists(&self) -> bool {
        self.entry_type() != EntryType::Unassigned
    }

    pub fn last_changed(&self) -> NetworkTime {
        unsafe { NetworkTime(sys::NT_GetEntryLastChange(self.handle)) }
    }

    pub fn name_bytes(&self) -> Vec<u8> {
        let mut len = 0;
        let char_ptr = unsafe { sys::NT_GetEntryName(self.handle, &mut len) };
        unsafe { ::std::slice::from_raw_parts(char_ptr, len).iter().map(|&ch| ch as u8).collect() }
    }

    pub fn name(&self) -> Option<String> {
        ::std::string::String::from_utf8(self.name_bytes()).ok()
    }

    pub fn set<V: Into<Value>>(&self, value: V) -> Result<(), EntryType> {
        if unsafe { sys::NT_SetEntryValue(self.handle, &value.into().to_nt_value(0)) != 0 } {
            Ok(())
        } else {
            // SetEntryValue returns false if there was a type mismatch, so we return what the type
            // of the current entry is.
            Err(EntryType::from(unsafe { sys::NT_GetEntryType(self.handle) }))
        }
    }

    pub fn edit<F: Fn(Value) -> Value>(&self, func: F) -> bool {
        self.value().map(func).map(|val| self.set(val)).is_some()
    }

    /// Get the value of this entry, if this entry does point to something.
    pub fn value(&self) -> Option<Value> {
        if !self.exists() { return None; }

        unsafe {
            let mut value = ::std::mem::zeroed();
            sys::NT_GetEntryValue(self.handle, &mut value);

            // No need to check if value's type is unassigned because we did that check up
            // at the top of the function.
            let val = match value.type_.into() {
                EntryType::Boolean => Value::Bool(value.data.v_boolean != 0),
                EntryType::Double => Value::Double(value.data.v_double as f64),
                EntryType::String => Value::String(NtString(value.data.v_string).as_str().to_owned()),
                EntryType::Raw => Value::Raw(NtString(value.data.v_string).as_bytes().to_owned()),

                // We have to write this out 3 times because bindgen generates 3 types here.
                // We could use a macro but *ehhhh*
                EntryType::BooleanArray => {
                    let bool_arr = value.data.arr_boolean;
                    let slice = ::std::slice::from_raw_parts(bool_arr.arr as *mut sys::NT_Bool, bool_arr.size);
                    Value::BoolArray(slice.iter().map(|&val| val != 0).collect())
                }

                EntryType::DoubleArray => {
                    let bool_arr = value.data.arr_double;
                    let slice = ::std::slice::from_raw_parts(bool_arr.arr as *mut f64, bool_arr.size);
                    Value::DoubleArray(slice.iter().map(|&val| val as f64).collect())
                }

                EntryType::StringArray => {
                    let bool_arr = value.data.arr_boolean;
                    let slice = ::std::slice::from_raw_parts(bool_arr.arr as *mut sys::NT_String, bool_arr.size);
                    Value::StringArray(slice.iter().map(|&val| NtString(val).as_str().to_owned()).collect())
                }

                // unassigned case
                EntryType::Unassigned => unreachable!(),
                EntryType::Rpc => unimplemented!(),
            };

            // We've copied all the data from the union in one way or another; we can dispose of it now
            sys::NT_DisposeValue(&mut value);

            Some(val)
        }
    }
}
