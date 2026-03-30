use crate::shell::BuiltinFn;

use std::collections::HashMap;

pub use super::builtins::{
    base::{clear, echo, exit, pwd},
    cat, cd, cp, list, mkdir, mv, rm,
};

pub fn get_builtins() -> HashMap<String, BuiltinFn> {
    HashMap::from([
        ("exit".to_string(), exit as BuiltinFn),
        ("echo".to_string(), echo as BuiltinFn),
        ("pwd".to_string(), pwd as BuiltinFn),
        ("clear".to_string(), clear as BuiltinFn),
        ("ls".to_string(), list::ls as BuiltinFn),
        ("cd".to_string(), cd::cd as BuiltinFn),
        ("cat".to_string(), cat::cat as BuiltinFn),
        ("cp".to_string(), cp::cp as BuiltinFn),
        ("rm".to_string(), rm::rm as BuiltinFn),
        ("mv".to_string(), mv::mv as BuiltinFn),
        ("mkdir".to_string(), mkdir::mkdir as BuiltinFn),
    ])
}
