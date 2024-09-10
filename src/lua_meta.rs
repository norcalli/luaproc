use std::path::PathBuf;

use proc_macro::Literal;
use quote::{format_ident, ToTokens};
use syn::{DeriveInput, Expr, LitStr};

use serde::Serialize;

use crate::lua::{attributes, Attr, Generics};

// structure holding attributes and generics for the whole struct or enum
#[derive(Debug, Serialize)]
pub(super) struct LuaMeta {
    // name of struct or enum
    ident: String,

    // List of outer attributes
    attributes: Vec<Attr>,

    // generics
    generics: Generics,
}

impl LuaMeta {
    #[allow(clippy::field_reassign_with_default)]
    pub(super) fn new(di: &DeriveInput) -> (Self, PathBuf) {
        let script_path = {
            let target_ident = format_ident!("luaproc");
            di.attrs
                .iter()
                .find_map(|attr| {
                    let ident = attr.meta.path().get_ident()?;
                    let _ = Some(()).filter(|()| *ident == target_ident)?;
                    let list = attr.meta.require_list().ok()?;
                    let expr: LitStr = list.parse_args().ok()?;
                    Some(expr.value())
                })
                .expect(r#"Failed to find script path. Specify #[luaproc("script_path")]"#)
        };
        (
            Self {
                ident: di.ident.to_string(),
                generics: {
                    let (impl_generics, ty_generics, where_clause) = di.generics.split_for_impl();
                    Generics {
                        r#impl: impl_generics.to_token_stream().to_string(),
                        r#type: ty_generics.to_token_stream().to_string(),
                        r#where: where_clause.to_token_stream().to_string(),
                    }
                },
                attributes: attributes(&di.attrs),
            },
            script_path.into(),
        )
    }

    // create Lua "meta" global variable
    // pub(super) fn lua_set_var(&self, lua: &MLua, globals: &Table<'_>) -> mlua::Result<()> {
    //     Lua::lua_set_var(lua, globals, self, "meta")
    // }
}
