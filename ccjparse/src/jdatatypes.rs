//! Module encapsules type definitions to represent JSON data types, which can be parsed from a
//! JSON-file.

#[derive(Debug, Clone)]
pub struct KeyValuePair<T> {
    name: String,
    value: T,
}

#[derive(Debug, Clone)]
pub enum JsonValueType {
    String,     // String
    Number,     // isize | f64
    Object,     // Object type
    Array,      // [??; ??] ??
    Boolean,    // bool
    Null,       // NullDef
}

/// New type definition to have a virtual JSON-Null-type.
#[derive(Debug, Clone)]
pub struct Null;
