use crate::syn::expand_cfg_if;
use crate::syn::find_cfg_items;
use crate::syn::find_cfg_mod;
use crate::transform::simplified_expr;

use codegen_cfg::ast::all as cfg_all;
use codegen_cfg::ast::any as cfg_any;
use codegen_cfg::ast::Expr as CfgExpr;

use std::collections::HashMap;
use std::fs;
use std::ops::Not as _;

use syn::File;
use syn::Ident;

use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use log::debug;
use regex::RegexSet;
use rust_utils::default::default;
use rust_utils::iter::map_collect_vec;

pub struct CfgItem {
    pub cfg: CfgExpr,
    pub name: String,
}

pub fn search(libc: impl AsRef<Utf8Path>, re: &RegexSet) -> Result<Vec<CfgItem>> {
    let items = search_items(libc.as_ref(), re)?;

    Ok(map_collect_vec(items, |(cfg, name)| CfgItem {
        cfg,
        name: name.to_string(),
    }))
}

fn search_items(libc: &Utf8Path, re: &RegexSet) -> Result<Vec<(CfgExpr, Ident)>> {
    let mut ctx = DfsContext { re, items: default() };

    {
        let dir = libc.join("src");
        let fs_path = libc.join("src/lib.rs");

        let ast = parse_file(&fs_path)?;
        for (cfg, name) in find_mod_in_file(&ast) {
            let fs_path = resolve_fs_path(&dir, &name);
            dfs(&mut ctx, &fs_path, &name, &cfg)?;
        }
    }

    debug!("aggregating items");
    Ok(aggregate_items(ctx.items))
}

fn parse_file(fs_path: &Utf8Path) -> Result<File> {
    let s = fs::read_to_string(fs_path)?;
    let mut ast = syn::parse_file(&s)?;
    expand_cfg_if(&mut ast);
    Ok(ast)
}

fn find_mod_in_file(ast: &File) -> Vec<(CfgExpr, Ident)> {
    let mut map: HashMap<Ident, Vec<CfgExpr>> = default();

    for (cfg, name) in find_cfg_mod(ast) {
        map.entry(name).or_default().push(cfg);
    }

    aggregate_items(map)
}

fn aggregate_items(map: HashMap<Ident, Vec<CfgExpr>>) -> Vec<(CfgExpr, Ident)> {
    let mut ans = map_collect_vec(map, |(name, exprs)| (simplified_expr(cfg_any(exprs)), name));
    ans.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));
    ans
}

fn resolve_fs_path(dir: &Utf8Path, mod_name: &Ident) -> Utf8PathBuf {
    let first = dir.join(format!("{}.rs", mod_name));
    let second = dir.join(format!("{}/mod.rs", mod_name));

    if first.exists() {
        assert!(second.exists().not());
        first
    } else {
        second
    }
}

struct DfsContext<'a> {
    re: &'a RegexSet,
    items: HashMap<Ident, Vec<CfgExpr>>,
}

fn dfs(ctx: &mut DfsContext, fs_path: &Utf8Path, mod_name: &Ident, mod_cfg: &CfgExpr) -> Result<()> {
    debug!("dfs: mod {mod_name:<16} at {fs_path:<80}: {mod_cfg}");

    let dir = fs_path.parent().unwrap();

    let ast = parse_file(fs_path)?;

    for (cfg, name) in find_cfg_items(&ast, ctx.re) {
        let item_cfg = join_item_cfg(mod_cfg, cfg);
        ctx.items.entry(name).or_default().push(item_cfg);
    }

    for (cfg, name) in find_mod_in_file(&ast) {
        let mod_name = &name;
        let fs_path = resolve_fs_path(dir, mod_name);
        let mod_cfg = join_item_cfg(mod_cfg, cfg);

        dfs(ctx, &fs_path, mod_name, &mod_cfg)?;
    }

    Ok(())
}

fn join_item_cfg(mod_cfg: &CfgExpr, item_cfg: CfgExpr) -> CfgExpr {
    simplified_expr(cfg_all((mod_cfg.clone(), item_cfg)))
}
