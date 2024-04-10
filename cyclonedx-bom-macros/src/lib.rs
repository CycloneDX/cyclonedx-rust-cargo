use std::error::Error as StdError;
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    fold::{self, Fold},
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Error, Expr, Item, Stmt,
};

#[derive(PartialEq, Eq)]
struct Version {
    major: usize,
    minor: usize,
}

impl FromStr for Version {
    type Err = Box<dyn StdError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major_str, minor_str) = s
            .split_once('.')
            .ok_or_else(|| Self::Err::from("missing `.`".to_owned()))?;

        Ok(Self {
            major: major_str.parse()?,
            minor: minor_str.parse()?,
        })
    }
}

impl Version {
    fn as_ident(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("v{}_{}", self.major, self.minor),
            Span::call_site(),
        )
    }
}

enum VersionReq {
    Any(Vec<Version>),
}

impl syn::parse::Parse for VersionReq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let versions = Punctuated::<syn::LitStr, Comma>::parse_terminated(input)?
            .into_iter()
            .map(|s| s.value().parse().map_err(|err| Error::new(s.span(), err)))
            .collect::<syn::Result<Vec<Version>>>()?;

        Ok(Self::Any(versions))
    }
}

impl VersionReq {
    fn matches(&self, version: &Version) -> bool {
        match self {
            VersionReq::Any(versions) => versions.contains(version),
        }
    }
}

struct VersionFilter {
    version: Version,
    error: Option<Error>,
}

impl VersionFilter {
    fn is_active(&mut self, attrs: &mut Vec<syn::Attribute>) -> bool {
        let mut matches = true;

        attrs.retain(|attr| {
            let path = attr.path();

            if path.is_ident("versioned") {
                match attr.parse_args::<VersionReq>() {
                    Ok(req) => matches = req.matches(&self.version),
                    Err(err) => match self.error.as_mut() {
                        Some(error) => error.combine(err),
                        None => self.error = Some(err),
                    },
                }

                false
            } else {
                true
            }
        });

        matches
    }

    fn filter_fields(
        &mut self,
        fields: Punctuated<syn::Field, Comma>,
    ) -> Punctuated<syn::Field, Comma> {
        fields
            .into_pairs()
            .filter_map(|mut pair| self.is_active(&mut pair.value_mut().attrs).then_some(pair))
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

    fn fold_stmt(&mut self, mut stmt: Stmt) -> Stmt {
        if let Stmt::Local(syn::Local { ref mut attrs, .. })
        | Stmt::Macro(syn::StmtMacro { ref mut attrs, .. }) = &mut stmt
        {
            if !self.is_active(attrs) {
                stmt = Stmt::Item(Item::Verbatim(TokenStream2::new()));
            }
        }

        fold::fold_stmt(self, stmt)
    }

    fn fold_expr(&mut self, mut expr: Expr) -> Expr {
        if let Expr::Array(syn::ExprArray { ref mut attrs, .. })
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
        | Expr::Yield(syn::ExprYield { ref mut attrs, .. }) = &mut expr
        {
            if !self.is_active(attrs) {
                expr = parse_quote!({});
            }
        }

        fold::fold_expr(self, expr)
    }

    fn fold_expr_struct(&mut self, mut expr: syn::ExprStruct) -> syn::ExprStruct {
        expr.fields = expr
            .fields
            .into_pairs()
            .filter_map(|mut pair| self.is_active(&mut pair.value_mut().attrs).then_some(pair))
            .collect();

        fold::fold_expr_struct(self, expr)
    }

    fn fold_expr_match(&mut self, mut expr: syn::ExprMatch) -> syn::ExprMatch {
        expr.arms.retain_mut(|arm| self.is_active(&mut arm.attrs));

        fold::fold_expr_match(self, expr)
    }

    fn fold_item(&mut self, mut item: Item) -> Item {
        if let Item::Const(syn::ItemConst { ref mut attrs, .. })
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
        | Item::Use(syn::ItemUse { ref mut attrs, .. }) = &mut item
        {
            if !self.is_active(attrs) {
                item = Item::Verbatim(TokenStream2::new());
            }
        }

        fold::fold_item(self, item)
    }

    fn fold_item_enum(&mut self, mut item: syn::ItemEnum) -> syn::ItemEnum {
        item.variants = item
            .variants
            .into_iter()
            .filter_map(|mut variant| self.is_active(&mut variant.attrs).then_some(variant))
            .collect();

        fold::fold_item_enum(self, item)
    }
}

fn helper(input: TokenStream, annotated_item: TokenStream) -> syn::Result<TokenStream2> {
    // This parses the module being annotated by the `#[versioned(..)]` attribute.
    let module = syn::parse::<syn::ItemMod>(annotated_item)
        .map_err(|err| Error::new(err.span(), format!("cannot parse module: {err}")))?;

    // This parses the versions passed to the attribute, e.g. the `"1.3"`
    // and `"1.4"`in `#[versioned("1.3", "1.4")]
    let versions =
        syn::parse::Parser::parse(Punctuated::<syn::LitStr, Comma>::parse_terminated, input)?
            .into_iter()
            .map(|s| s.value().parse().map_err(|err| Error::new(s.span(), err)))
            .collect::<syn::Result<Vec<Version>>>()?;

    let content = module
        .content
        .as_ref()
        .ok_or_else(|| Error::new(module.ident.span(), "found module without content"))?;

    let mut tokens = TokenStream2::new();

    for version in versions {
        let mod_vis = &module.vis;
        let mod_ident = version.as_ident();

        let items = content.1.clone();

        let mut folded_items = Vec::new();

        let mut filter = VersionFilter {
            version,
            error: None,
        };

        for item in items {
            folded_items.push(filter.fold_item(item));
            if let Some(error) = filter.error {
                return Err(error);
            }
        }

        tokens.extend(quote! {
             #mod_vis mod #mod_ident {
                #(#folded_items)*
            }
        })
    }

    Ok(tokens)
}

/// A `cfg`-like attribute macro to generate versioned modules.
///
/// This macro allows to duplicate a module by providing version numbers to the
/// macro itself. For example:
/// ```rust
/// use cyclonedx_bom_macros::versioned;
///
/// #[versioned("1.0", "2.0")]
/// mod base {
///    pub(super) struct Foo;
/// }
/// ```
/// Will generate two modules: `v1_0` and `v2_0`, where each one of them
/// contains the definition of `Foo`:
/// ```rust
/// mod v1_0 {
///    pub(super) struct Foo;
/// }
///
/// mod v2_0 {
///    pub(super) struct Foo;
/// }
/// ```
/// Additionally the macro can be used to gate definitions and expressions
/// behind a specific version, very much like the `cfg` attribute. Based on the
/// previous example:
/// ```rust
/// use cyclonedx_bom_macros::versioned;
///
/// #[versioned("1.0", "2.0")]
/// mod base {
///    pub(super) struct Foo;
///
///    #[versioned("2.0")]
///    pub(super) struct Bar;
/// }
/// ```
/// The following code will be generated:
/// ```rust
/// mod v1_0 {
///    pub(super) struct Foo;
/// }
///
/// mod v2_0 {
///    pub(super) struct Foo;
///    pub(super) struct Bar;
/// }
/// ```
/// Note that `Bar` only exists inside the `v2_0` module. Note that the
/// `versioned` attribute annotating the module defines the versions that will be
/// used to generate the modules and the attribute annotating the `Bar` definition
/// states that this definition will only appear on the `2.0` module.
///
/// Check the test folder for more usage examples.
#[proc_macro_attribute]
pub fn versioned(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    match helper(input, annotated_item) {
        Ok(tokens) => tokens,
        Err(err) => Error::new(
            err.span(),
            format!("{err} while using the `#[versioned]` macro"),
        )
        .into_compile_error(),
    }
    .into()
}
