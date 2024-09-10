use serde::Serialize;
use syn::DataStruct;

use crate::{
    lua::{fields, Field},
    lua_meta::LuaMeta,
};

/// Defines the structures which are serialized and then passed to Lua script
/// as global tables.
#[derive(Debug, Serialize)]
pub(super) struct LuaStruct {
    // List of structure fields
    fields: Vec<Field>,

    // meta information
    pub(super) meta: LuaMeta,
}

impl LuaStruct {
    #[allow(clippy::field_reassign_with_default)]
    pub(super) fn new(meta: LuaMeta, ds: &DataStruct) -> Self {
        // this will act as the interface between Rust & Lua
        Self {
            fields: fields(&ds.fields),
            meta,
        }
    }
}
