use crate::transform::simplified_expr;

use codegen_cfg::ast::all as cfg_all;
use codegen_cfg::ast::Expr as CfgExpr;

use std::borrow::Cow;
use std::ops::Not;

use syn::Attribute;
use syn::Block;
use syn::File;
use syn::Ident;
use syn::Item;
use syn::ItemMacro;
use syn::Stmt;

use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_quote;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::Token;

use rust_utils::default::default;
use rust_utils::iter::map_collect_vec;
use rust_utils::vec::VecExt;

use log::debug;
use proc_macro2::TokenStream;
use quote::quote;
use regex::RegexSet;

pub fn expand_cfg_if(ast: &mut File) {
    ExpandCfgIf.visit_file_mut(ast);
}

struct ExpandCfgIf;

impl ExpandCfgIf {
    fn matches(ast: &syn::Item) -> bool {
        if let Item::Macro(ast) = ast {
            if let Some(ident) = ast.mac.path.get_ident() {
                if ident == "cfg_if" {
                    return true;
                }
            }
        }
        false
    }

    fn generate_branch_cfg(prev: &[&TokenStream], cur: Option<&TokenStream>) -> TokenStream {
        match cur {
            Some(cur) => {
                if prev.is_empty() {
                    cur.clone()
                } else {
                    quote! { all(not(any(#(#prev),*)), #cur) }
                }
            }
            None => {
                assert!(prev.is_empty().not());
                quote! { not(any(#(#prev),*)) }
            }
        }
    }

    fn get_item_attrs_mut(item: &mut Item) -> &mut Vec<Attribute> {
        match item {
            Item::Const(x) => &mut x.attrs,
            Item::Enum(x) => &mut x.attrs,
            Item::ExternCrate(x) => &mut x.attrs,
            Item::Fn(x) => &mut x.attrs,
            Item::ForeignMod(x) => &mut x.attrs,
            Item::Impl(x) => &mut x.attrs,
            Item::Macro(x) => &mut x.attrs,
            Item::Mod(x) => &mut x.attrs,
            Item::Static(x) => &mut x.attrs,
            Item::Struct(x) => &mut x.attrs,
            Item::Trait(x) => &mut x.attrs,
            Item::TraitAlias(x) => &mut x.attrs,
            Item::Type(x) => &mut x.attrs,
            Item::Union(x) => &mut x.attrs,
            Item::Use(x) => &mut x.attrs,
            _ => unimplemented!(),
        }
    }

    fn apply(cfg: &TokenStream, block: Block) -> Vec<Item> {
        let mut ans: Vec<Item> = default();
        for stmt in block.stmts {
            match stmt {
                Stmt::Item(mut item) => {
                    let attrs = Self::get_item_attrs_mut(&mut item);
                    let cfg: Attribute = parse_quote! { #[cfg(#cfg)] };
                    attrs.insert(0, cfg);
                    ans.push(item);
                }
                Stmt::Macro(stmt_macro) => {
                    let mut item_macro = ItemMacro {
                        attrs: stmt_macro.attrs,
                        ident: None,
                        mac: stmt_macro.mac,
                        semi_token: stmt_macro.semi_token,
                    };
                    let cfg: Attribute = parse_quote! { #[cfg(#cfg)] };
                    item_macro.attrs.insert(0, cfg);
                    ans.push(Item::Macro(item_macro));
                }
                Stmt::Local(_) => unimplemented!(),
                Stmt::Expr(_, _) => unimplemented!(),
            }
        }
        ans
    }

    fn expand(ast: &Item) -> Vec<Item> {
        let Item::Macro(ast) = ast else { panic!() };
        let macro_: CfgIf = syn::parse2(ast.mac.tokens.clone()).expect("ill-formed cfg_if");

        debug!("expanding cfg_if");

        let mut items: Vec<Item> = default();
        let mut prev: Vec<&TokenStream> = default();

        {
            let cfg = Self::generate_branch_cfg(&prev, Some(&macro_.if_.cfg));
            items.extend(Self::apply(&cfg, macro_.if_.block));
            prev.push(&macro_.if_.cfg);
        }

        for else_if in &macro_.else_if {
            let cfg = Self::generate_branch_cfg(&prev, Some(&else_if.cfg));
            items.extend(Self::apply(&cfg, else_if.block.clone()));
            prev.push(&else_if.cfg);
        }

        if let Some(else_) = macro_.else_ {
            let cfg = Self::generate_branch_cfg(&prev, None);
            items.extend(Self::apply(&cfg, else_));
        }

        items
    }
}

impl VisitMut for ExpandCfgIf {
    fn visit_file_mut(&mut self, ast: &mut File) {
        let mut i = 0;
        while i < ast.items.len() {
            if Self::matches(&ast.items[i]) {
                let items = Self::expand(&ast.items[i]);
                ast.items.remove(i);
                ast.items.insert_vec(i, items);
            } else {
                i += 1;
            }
        }
    }
}

#[derive(Debug)]
struct CfgIf {
    if_: CfgIfBranch,
    else_if: Vec<CfgIfBranch>,
    else_: Option<Block>,
}

#[derive(Debug)]
struct CfgIfBranch {
    cfg: TokenStream,
    block: Block,
}

impl Parse for CfgIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        let if_: CfgIfBranch = input.parse()?;

        let mut else_if: Vec<CfgIfBranch> = default();

        while input.peek(Token![else]) && input.peek2(Token![if]) {
            input.parse::<Token![else]>()?;
            input.parse::<Token![if]>()?;
            else_if.push(input.parse()?);
        }

        let else_ = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        assert!(input.is_empty());

        Ok(Self { if_, else_if, else_ })
    }
}

impl Parse for CfgIfBranch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;

        assert_eq!(attrs.len(), 1);
        let attr = attrs.pop().unwrap();

        assert!(attr.meta.path().is_ident("cfg"));

        let cfg = &attr.meta.require_list()?.tokens;
        let cfg = quote! { all(#cfg) };

        let block: Block = input.parse()?;

        Ok(Self { cfg, block })
    }
}

pub fn find_cfg_mod(ast: &File) -> Vec<(CfgExpr, Ident)> {
    let mut find_mod = FindMods { ans: Vec::new() };
    find_mod.visit_file(ast);
    find_mod.ans
}

#[derive(Debug)]
pub struct CfgMod {
    pub cfg: Option<CfgExpr>,
    pub name: Ident,
}

struct FindMods {
    ans: Vec<(CfgExpr, Ident)>,
}

impl<'ast> Visit<'ast> for FindMods {
    fn visit_item_mod(&mut self, ast: &'ast syn::ItemMod) {
        debug!("visit_item_mod: {:?}", ast.ident);

        assert!(ast.content.is_none());

        let cfg = find_cfg_in_attrs(&ast.attrs);
        let name = ast.ident.clone();

        self.ans.push((cfg, name));
    }
}

fn parse_cfg(tokens: &TokenStream) -> CfgExpr {
    let tokens = tokens.to_string();
    let result = codegen_cfg::parsing::parse(&tokens);
    match result {
        Ok(expr) => expr,
        Err(e) => panic!(
            "ill-formed cfg: tokens = {tokens:?}, e.location = {}, e.input = {:?}\nerror backtrace:\n{}\n\n",
            e.location, e.input, e.backtrace
        ),
    }
}

fn find_cfg_in_attrs(attrs: &[Attribute]) -> CfgExpr {
    let cfg_attrs = attrs
        .iter()
        .filter(|attr| attr.meta.path().is_ident("cfg"))
        .map(|attr| &attr.meta.require_list().unwrap().tokens);

    let cfg_exprs = map_collect_vec(cfg_attrs, parse_cfg);

    simplified_expr(cfg_all(cfg_exprs))
}

pub fn find_cfg_items(ast: &File, re: &RegexSet) -> Vec<(CfgExpr, Ident)> {
    let mut find_items = FindItems { re, ans: Vec::new() };
    find_items.visit_file(ast);
    find_items.ans
}

struct FindItems<'a> {
    re: &'a RegexSet,
    ans: Vec<(CfgExpr, Ident)>,
}

impl FindItems<'_> {
    fn expand_item_group(&mut self, tokens: &TokenStream) {
        let stmts: Vec<Stmt> = parse_quote! { #tokens };
        for stmt in stmts {
            let Stmt::Item(item) = stmt else { panic!() };
            self.visit_item(&item);
        }
    }
}

macro_rules! push {
    ($self:expr, $ast:expr) => {{
        let ast = $ast;
        if let syn::Visibility::Public(_) = ast.vis {
            if $self.re.is_match(&ast.ident.to_string()) {
                let cfg = find_cfg_in_attrs(&ast.attrs);
                let name = ast.ident.clone();
                $self.ans.push((cfg, name));
            }
        }
    }};
    ($self:expr, $ast:expr, $ident:expr) => {{
        let ast = $ast;
        let ident = $ident;
        if let syn::Visibility::Public(_) = ast.vis {
            if $self.re.is_match(&ident.to_string()) {
                let cfg = find_cfg_in_attrs(&ast.attrs);
                let name = ident.clone();
                $self.ans.push((cfg, name));
            }
        }
    }};
}

impl<'ast> Visit<'ast> for FindItems<'_> {
    fn visit_item(&mut self, ast: &'ast Item) {
        match ast {
            Item::Const(ast) => push!(self, ast),
            Item::Struct(ast) => push!(self, ast),
            Item::Union(ast) => push!(self, ast),
            Item::Fn(ast) => push!(self, ast, &ast.sig.ident),
            Item::Type(ast) => push!(self, ast),
            Item::Enum(ast) => push!(self, ast),
            Item::Static(ast) => push!(self, ast),
            //
            Item::Mod(ast) => assert!(ast.content.is_none()),
            Item::Macro(ast) => self.visit_item_macro(ast),
            Item::ForeignMod(ast) => self.visit_item_foreign_mod(ast),
            //
            Item::Impl(_) => {}
            Item::ExternCrate(_) => {}
            Item::Trait(_) => {}
            Item::TraitAlias(_) => {}
            Item::Use(_) => {}
            //
            Item::Verbatim(_) => panic!(),
            _ => panic!(),
        }
    }

    fn visit_item_macro(&mut self, ast: &'ast ItemMacro) {
        if ast.ident.is_some() {
            return;
        }
        let ident = ast.mac.path.get_ident().unwrap();

        if ident == "static_assert_eq" {
            return;
        }

        let const_hack = ["f", "safe_f", "const_fn"];

        let mut tokens = Cow::Borrowed(&ast.mac.tokens);

        if const_hack.iter().any(|x| ident == x) {
            let code = ast.mac.tokens.to_string().replace("{ const }", "const");
            tokens = Cow::Owned(code.parse::<TokenStream>().unwrap());
        }

        let item_group_macros = [
            "s",
            "s_no_extra_traits",
            "e",
            "s_paren",
            "align_const",
            "deprecated_mach",
            "expand_align",
            "f",
            "safe_f",
            "const_fn",
        ];

        if item_group_macros.iter().any(|x| ident == x) {
            debug!("expanding item group: {ident}!");
            self.expand_item_group(&tokens);
            return;
        }

        log::warn!("unknown macro: {ident}");
    }

    fn visit_foreign_item(&mut self, ast: &'ast syn::ForeignItem) {
        match ast {
            syn::ForeignItem::Fn(ast) => push!(self, ast, &ast.sig.ident),
            syn::ForeignItem::Static(ast) => push!(self, ast),
            syn::ForeignItem::Type(ast) => push!(self, ast),
            syn::ForeignItem::Macro(_) => unimplemented!(),
            syn::ForeignItem::Verbatim(_) => panic!(),
            _ => panic!(),
        }
    }
}
