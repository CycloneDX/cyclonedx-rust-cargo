use std::{error::Error, str::FromStr};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    fold::{self, Fold},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Expr, Item,
};

#[derive(PartialEq, Eq)]
struct Version {
    major: usize,
    minor: usize,
}

impl FromStr for Version {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major_str, minor_str) = s
            .split_once('.')
            .ok_or_else(|| Self::Err::from("versions must have a `.`".to_owned()))?;

        Ok(Self {
            major: major_str.parse()?,
            minor: minor_str.parse()?,
        })
    }
}

impl Version {
    fn extract_from_attrs(attrs: &mut Vec<syn::Attribute>) -> Option<Self> {
        let mut version = None;

        attrs.retain(|attr| {
            let path = attr.path();

            if path.is_ident("versioned") {
                version = Some(
                    attr.parse_args::<syn::LitStr>()
                        .expect("expected a string literal")
                        .value()
                        .parse()
                        .expect("cannot parse version"),
                );

                false
            } else {
                true
            }
        });

        version
    }

    fn as_ident(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("v{}_{}", self.major, self.minor),
            Span::call_site(),
        )
    }
}

struct VersionFilter {
    version: Version,
}

impl VersionFilter {
    fn matches(&self, found_version: &Version) -> bool {
        &self.version == found_version
    }

    fn filter_fields(
        &self,
        fields: Punctuated<syn::Field, Comma>,
    ) -> Punctuated<syn::Field, Comma> {
        fields
            .into_pairs()
            .filter_map(
                |mut pair| match Version::extract_from_attrs(&mut pair.value_mut().attrs) {
                    Some(version) => self.matches(&version).then_some(pair),
                    None => Some(pair),
                },
            )
            .collect()
    }
}

impl Fold for VersionFilter {
    fn fold_fields_named(&mut self, mut fields: syn::FieldsNamed) -> syn::FieldsNamed {
        fields.named = self.filter_fields(fields.named);
        fields
    }

    fn fold_fields_unnamed(&mut self, mut fields: syn::FieldsUnnamed) -> syn::FieldsUnnamed {
        fields.unnamed = self.filter_fields(fields.unnamed);
        fields
    }

    fn fold_stmt(&mut self, mut stmt: syn::Stmt) -> syn::Stmt {
        match stmt {
            syn::Stmt::Local(syn::Local { ref mut attrs, .. })
            | syn::Stmt::Macro(syn::StmtMacro { ref mut attrs, .. }) => {
                if let Some(version) = Version::extract_from_attrs(attrs) {
                    if !self.matches(&version) {
                        stmt = parse_quote!({};);
                    }
                }
            }
            _ => {}
        }

        fold::fold_stmt(self, stmt)
    }

    fn fold_expr(&mut self, mut expr: Expr) -> Expr {
        match &mut expr {
            Expr::Array(syn::ExprArray { ref mut attrs, .. })
            | Expr::Assign(syn::ExprAssign { ref mut attrs, .. })
            | Expr::Async(syn::ExprAsync { ref mut attrs, .. })
            | Expr::Await(syn::ExprAwait { ref mut attrs, .. })
            | Expr::Binary(syn::ExprBinary { ref mut attrs, .. })
            | Expr::Block(syn::ExprBlock { ref mut attrs, .. })
            | Expr::Break(syn::ExprBreak { ref mut attrs, .. })
            | Expr::Call(syn::ExprCall { ref mut attrs, .. })
            | Expr::Cast(syn::ExprCast { ref mut attrs, .. })
            | Expr::Closure(syn::ExprClosure { ref mut attrs, .. })
            | Expr::Const(syn::ExprConst { ref mut attrs, .. })
            | Expr::Continue(syn::ExprContinue { ref mut attrs, .. })
            | Expr::Field(syn::ExprField { ref mut attrs, .. })
            | Expr::ForLoop(syn::ExprForLoop { ref mut attrs, .. })
            | Expr::Group(syn::ExprGroup { ref mut attrs, .. })
            | Expr::If(syn::ExprIf { ref mut attrs, .. })
            | Expr::Index(syn::ExprIndex { ref mut attrs, .. })
            | Expr::Infer(syn::ExprInfer { ref mut attrs, .. })
            | Expr::Let(syn::ExprLet { ref mut attrs, .. })
            | Expr::Lit(syn::ExprLit { ref mut attrs, .. })
            | Expr::Loop(syn::ExprLoop { ref mut attrs, .. })
            | Expr::Macro(syn::ExprMacro { ref mut attrs, .. })
            | Expr::Match(syn::ExprMatch { ref mut attrs, .. })
            | Expr::MethodCall(syn::ExprMethodCall { ref mut attrs, .. })
            | Expr::Paren(syn::ExprParen { ref mut attrs, .. })
            | Expr::Path(syn::ExprPath { ref mut attrs, .. })
            | Expr::Range(syn::ExprRange { ref mut attrs, .. })
            | Expr::Reference(syn::ExprReference { ref mut attrs, .. })
            | Expr::Repeat(syn::ExprRepeat { ref mut attrs, .. })
            | Expr::Return(syn::ExprReturn { ref mut attrs, .. })
            | Expr::Struct(syn::ExprStruct { ref mut attrs, .. })
            | Expr::Try(syn::ExprTry { ref mut attrs, .. })
            | Expr::TryBlock(syn::ExprTryBlock { ref mut attrs, .. })
            | Expr::Tuple(syn::ExprTuple { ref mut attrs, .. })
            | Expr::Unary(syn::ExprUnary { ref mut attrs, .. })
            | Expr::Unsafe(syn::ExprUnsafe { ref mut attrs, .. })
            | Expr::While(syn::ExprWhile { ref mut attrs, .. })
            | Expr::Yield(syn::ExprYield { ref mut attrs, .. }) => {
                if let Some(version) = Version::extract_from_attrs(attrs) {
                    if !self.matches(&version) {
                        expr = parse_quote!({});
                    }
                }
            }
            _ => {}
        }

        fold::fold_expr(self, expr)
    }

    fn fold_expr_struct(&mut self, mut expr: syn::ExprStruct) -> syn::ExprStruct {
        expr.fields = expr
            .fields
            .into_pairs()
            .filter_map(
                |mut pair| match Version::extract_from_attrs(&mut pair.value_mut().attrs) {
                    Some(version) => self.matches(&version).then_some(pair),
                    None => Some(pair),
                },
            )
            .collect();

        fold::fold_expr_struct(self, expr)
    }

    fn fold_expr_match(&mut self, mut expr: syn::ExprMatch) -> syn::ExprMatch {
        expr.arms
            .retain_mut(|arm| match Version::extract_from_attrs(&mut arm.attrs) {
                Some(version) => self.matches(&version),
                None => true,
            });

        fold::fold_expr_match(self, expr)
    }

    fn fold_item(&mut self, mut item: Item) -> Item {
        match item {
            Item::Const(syn::ItemConst { ref mut attrs, .. })
            | Item::Enum(syn::ItemEnum { ref mut attrs, .. })
            | Item::ExternCrate(syn::ItemExternCrate { ref mut attrs, .. })
            | Item::Fn(syn::ItemFn { ref mut attrs, .. })
            | Item::ForeignMod(syn::ItemForeignMod { ref mut attrs, .. })
            | Item::Impl(syn::ItemImpl { ref mut attrs, .. })
            | Item::Macro(syn::ItemMacro { ref mut attrs, .. })
            | Item::Mod(syn::ItemMod { ref mut attrs, .. })
            | Item::Static(syn::ItemStatic { ref mut attrs, .. })
            | Item::Struct(syn::ItemStruct { ref mut attrs, .. })
            | Item::Trait(syn::ItemTrait { ref mut attrs, .. })
            | Item::TraitAlias(syn::ItemTraitAlias { ref mut attrs, .. })
            | Item::Type(syn::ItemType { ref mut attrs, .. })
            | Item::Union(syn::ItemUnion { ref mut attrs, .. })
            | Item::Use(syn::ItemUse { ref mut attrs, .. }) => {
                if let Some(version) = Version::extract_from_attrs(attrs) {
                    if !self.matches(&version) {
                        item = parse_quote!(
                            use {};
                        );
                    }
                }
            }
            _ => {}
        }

        fold::fold_item(self, item)
    }
}

#[proc_macro_attribute]
pub fn versioned(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    // This parses the module being annotated by the `#[versioned(..)]` attribute.
    let module = parse_macro_input!(annotated_item as syn::ItemMod);

    // This parses the versions passed to the attribute, e.g. the `"1.3"`
    // and `"1.4"`in `#[versioned("1.3", "1.4")]
    let versions: Vec<Version> =
        parse_macro_input!(input with Punctuated::<syn::LitStr, Comma>::parse_terminated)
            .into_iter()
            .map(|s| s.value().parse().expect("cannot parse version"))
            .collect();

    let mut tokens = proc_macro2::TokenStream::new();

    for version in versions {
        let mod_vis = &module.vis;
        let mod_ident = version.as_ident();

        let (_, items) = module.content.clone().unwrap();

        let mut folded_items = Vec::new();

        let mut filter = VersionFilter { version };

        for item in items {
            folded_items.push(filter.fold_item(item));
        }

        tokens.extend(quote! {
             #mod_vis mod #mod_ident {
                #(#folded_items)*
            }
        })
    }

    tokens.into()
}
