use quote::ToTokens;
use serde::Serialize;
use syn::DataEnum;

use crate::{
    lua::{attributes, fields, Field, Variant},
    lua_meta::LuaMeta,
};

/// Defines the structures which are serialized and then passed to Lua script
/// as global tables.
#[derive(Debug, Serialize)]
pub(super) struct LuaEnum {
    // List of structure fields
    variants: Vec<Variant>,

    // list of fields
    fields: Vec<Field>,

    pub(super) meta: LuaMeta,
}

impl LuaEnum {
    #[allow(clippy::field_reassign_with_default)]
    pub(super) fn new(meta: LuaMeta, ds: &DataEnum) -> Self {
        // this will act as the interface between Rust & Lua
        let mut variants = vec![];

        // lookup each variant
        for v in &ds.variants {
            variants.push(Variant {
                attributes: attributes(&v.attrs),
                name: v.ident.to_string(),
                fields: match &v.fields {
                    syn::Fields::Named(fnamed) => fields(&fnamed.named),
                    syn::Fields::Unnamed(funnamed) => fields(&funnamed.unnamed),
                    syn::Fields::Unit => vec![],
                },
                discriminant: if let Some(disc) = &v.discriminant {
                    Some(disc.1.to_token_stream().to_string())
                } else {
                    None
                },
                style: match &v.fields {
                    syn::Fields::Unnamed(_) => "tuple",
                    syn::Fields::Named(_) => "struct",
                    syn::Fields::Unit => "unit",
                },
                is_tuple: match &v.fields {
                    syn::Fields::Unnamed(_) => true,
                    syn::Fields::Named(_) | syn::Fields::Unit => false,
                },
                is_unit: match &v.fields {
                    syn::Fields::Unit => true,
                    syn::Fields::Named(_) | syn::Fields::Unnamed(_) => false,
                },
            });
        }

        LuaEnum {
            variants,
            fields: vec![],
            meta,
        }
    }
}
