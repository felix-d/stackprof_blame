//! Generated bindings for profile.proto from the pprof profiling tool
//! https://github.com/google/pprof

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Profile {
    #[prost(message, repeated, tag = "1")]
    pub sample_type: ::prost::alloc::vec::Vec<ValueType>,

    #[prost(message, repeated, tag = "2")]
    pub sample: ::prost::alloc::vec::Vec<Sample>,

    #[prost(message, repeated, tag = "3")]
    pub mapping: ::prost::alloc::vec::Vec<Mapping>,

    #[prost(message, repeated, tag = "4")]
    pub location: ::prost::alloc::vec::Vec<Location>,

    #[prost(message, repeated, tag = "5")]
    pub function: ::prost::alloc::vec::Vec<Function>,

    #[prost(string, repeated, tag = "6")]
    pub string_table: ::prost::alloc::vec::Vec<String>,

    #[prost(int64, tag = "7")]
    pub drop_frames: i64,

    #[prost(int64, tag = "8")]
    pub keep_frames: i64,

    #[prost(int64, tag = "9")]
    pub time_nanos: i64,

    #[prost(int64, tag = "10")]
    pub duration_nanos: i64,

    #[prost(string, tag = "11")]
    pub period_type: ::prost::alloc::string::String,

    #[prost(int64, tag = "12")]
    pub period: i64,

    #[prost(int64, tag = "13")]
    pub comment: i64,

    #[prost(int64, tag = "14")]
    pub default_sample_type: i64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueType {
    #[prost(int64, tag = "1")]
    pub r#type: i64,

    #[prost(int64, tag = "2")]
    pub unit: i64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sample {
    #[prost(uint64, repeated, tag = "1")]
    pub location_id: ::prost::alloc::vec::Vec<u64>,

    #[prost(int64, repeated, tag = "2")]
    pub value: ::prost::alloc::vec::Vec<i64>,

    #[prost(int64, repeated, tag = "3")]
    pub label: ::prost::alloc::vec::Vec<i64>,

    #[prost(int64, repeated, tag = "4")]
    pub num_label: ::prost::alloc::vec::Vec<i64>,

    #[prost(int64, repeated, tag = "5")]
    pub num_unit: ::prost::alloc::vec::Vec<i64>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mapping {
    #[prost(uint64, tag = "1")]
    pub id: u64,

    #[prost(uint64, tag = "2")]
    pub memory_start: u64,

    #[prost(uint64, tag = "3")]
    pub memory_limit: u64,

    #[prost(uint64, tag = "4")]
    pub file_offset: u64,

    #[prost(int64, tag = "5")]
    pub filename: i64,

    #[prost(int64, tag = "6")]
    pub build_id: i64,

    #[prost(bool, tag = "7")]
    pub has_functions: bool,

    #[prost(bool, tag = "8")]
    pub has_filenames: bool,

    #[prost(bool, tag = "9")]
    pub has_line_numbers: bool,

    #[prost(bool, tag = "10")]
    pub has_inline_frames: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Location {
    #[prost(uint64, tag = "1")]
    pub id: u64,

    #[prost(uint64, tag = "2")]
    pub mapping_id: u64,

    #[prost(uint64, tag = "3")]
    pub address: u64,

    #[prost(message, repeated, tag = "4")]
    pub line: ::prost::alloc::vec::Vec<Line>,

    #[prost(bool, tag = "5")]
    pub is_folded: bool,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Line {
    #[prost(uint64, tag = "1")]
    pub function_id: u64,

    #[prost(int64, tag = "2")]
    pub line: i64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Function {
    #[prost(uint64, tag = "1")]
    pub id: u64,

    #[prost(int64, tag = "2")]
    pub name: i64,

    #[prost(int64, tag = "3")]
    pub system_name: i64,

    #[prost(int64, tag = "4")]
    pub filename: i64,

    #[prost(int64, tag = "5")]
    pub start_line: i64,
}
