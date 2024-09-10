use std::{fs, path::Path};

use mlua::{Lua as MLua, LuaSerdeExt, Table};
use quote::ToTokens;
use serde::Serialize;
use syn::{punctuated::Punctuated, Attribute, Expr, Token};

// definition of all useful structs for building global variable
#[derive(Debug, Serialize)]
pub(super) struct Field {
    pub(super) name: String,
    pub(super) r#type: String,

    // List of inner attributes
    pub(super) attributes: Vec<Attr>,
}

#[derive(Debug, Serialize)]
pub(super) struct Variant {
    pub(super) name: String,
    // pub(super) ty: String,

    // List of inner attributes
    pub(super) attributes: Vec<Attr>,
    pub(super) is_tuple: bool,
    pub(crate) is_unit: bool,

    pub(super) discriminant: Option<String>,
    pub(super) fields: Vec<Field>,
    pub(crate) style: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AttrStyle {
    Path,
    List,
    Value,
}

#[derive(Debug, Serialize)]
pub(super) struct Attr {
    pub(super) name: Option<String>,
    pub(super) inner: String,
    pub(super) outer: String,
    pub(super) style: AttrStyle,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct Generics {
    pub(super) r#impl: String,
    pub(super) r#type: String,
    pub(super) r#where: String,
}

pub(super) struct Lua;

impl Lua {
    //───────────────────────────────────────────────────────────────────────────────────
    // inject Rust struct or enum as a Lua global variable
    //───────────────────────────────────────────────────────────────────────────────────
    pub(super) fn lua_set_var<T: Serialize>(
        lua: &MLua,
        globals: &Table<'_>,
        glob: &T,
        var: &str,
    ) -> mlua::Result<()> {
        let lua_var = lua.to_value(glob)?;
        globals.set(var, lua_var)
    }

    //───────────────────────────────────────────────────────────────────────────────────
    // execute Lua code from the source file
    //───────────────────────────────────────────────────────────────────────────────────
    pub(super) fn lua_exec_code(lua: &MLua, path: impl AsRef<Path>) -> mlua::Result<()> {
        let path = path.as_ref();
        // open source file
        let lua_code = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("can't open Lua source file {path}", path = path.display()));

        let res = lua.load(lua_code).exec();
        if let Err(e) = res {
            // its useful to get script name for debugging
            panic!("error in Lua script {path:?}: {e}", path = path.display())
        } else {
            res
        }
    }
}

//───────────────────────────────────────────────────────────────────────────────────
// a helper function for grabing field attributes
//───────────────────────────────────────────────────────────────────────────────────
#[allow(clippy::field_reassign_with_default)]
pub(super) fn attributes<'a, T>(value: T) -> Vec<Attr>
where
    T: IntoIterator<Item = &'a Attribute>,
{
    let mut v: Vec<Attr> = Vec::new();

    for attr in value {
        let name = attr.meta.path().get_ident().map(|ident| ident.to_string());

        let outer = attr.meta.to_token_stream().to_string();
        let inner = match &attr.meta {
            syn::Meta::Path(p) => p.to_token_stream().to_string(),
            syn::Meta::List(l) => {
                let exprs = l
                    .parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                    .expect("meta list parse args");
                exprs.to_token_stream().to_string()
            }
            syn::Meta::NameValue(n) => n.value.to_token_stream().to_string(),
        };
        let style = match &attr.meta {
            syn::Meta::Path(_) => AttrStyle::Path,
            syn::Meta::List(_) => AttrStyle::List,
            syn::Meta::NameValue(_) => AttrStyle::Value,
        };

        v.push(Attr {
            name,
            inner,
            outer,
            style,
        });
    }

    v
}

//───────────────────────────────────────────────────────────────────────────────────
// helper function to get fields and its features
//───────────────────────────────────────────────────────────────────────────────────
pub(super) fn fields<'a, T>(value: T) -> Vec<Field>
where
    T: IntoIterator<Item = &'a syn::Field>,
{
    let mut v: Vec<Field> = Vec::new();

    for (i, f) in value.into_iter().enumerate() {
        v.push(Field {
            name: f
                .ident
                .as_ref()
                .map(syn::Ident::to_string)
                .unwrap_or_else(|| format!("_{i}")),
            r#type: f.ty.to_token_stream().to_string(),
            attributes: attributes(&f.attrs),
        });
    }

    v
}
