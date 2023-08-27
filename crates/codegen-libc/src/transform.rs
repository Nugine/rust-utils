use codegen_cfg::ast::*;
use codegen_cfg::bool_logic::transform::*;
use codegen_cfg::bool_logic::visit_mut::*;
use rust_utils::iter::filter_map_collect_vec;
use rust_utils::iter::map_collect_vec;
use rust_utils::vec::VecExt;

use std::cmp::Ordering::{self, *};
use std::mem;

use log::debug;
use log::trace;

pub fn simplified_expr(x: impl Into<Expr>) -> Expr {
    let mut x = x.into();

    debug!("input:                              {x}");

    UnifyTargetFamily.visit_mut_expr(&mut x);
    trace!("after  UnifyTargetFamily:           {x}");

    for _ in 0..3 {
        FlattenSingle.visit_mut_expr(&mut x);
        trace!("after  FlattenSingle:               {x}");

        FlattenNestedList.visit_mut_expr(&mut x);
        trace!("after  FlattenNestedList:           {x}");

        DedupList.visit_mut_expr(&mut x);
        trace!("after  DedupList:                   {x}");

        EvalConst.visit_mut_expr(&mut x);
        trace!("after  EvalConst:                   {x}");

        SimplifyNestedList.visit_mut_expr(&mut x);
        trace!("after  SimplifyNestedList:          {x}");

        MergeAllOfNotAny.visit_mut_expr(&mut x);
        trace!("after  MergeAllOfNotAny:            {x}");

        SimplifyAllNotAny.visit_mut_expr(&mut x);
        trace!("after  SimplifyAllNotAny:           {x}");

        MergeAllOfAny.visit_mut_expr(&mut x);
        trace!("after  MergeAllOfAny:               {x}");

        ImplyByKey.visit_mut_expr(&mut x);
        trace!("after  ImplyByKey:                  {x}");

        SuppressTargetFamily.visit_mut_expr(&mut x);
        trace!("after  SuppressTargetFamily:        {x}");

        EvalConst.visit_mut_expr(&mut x);
        trace!("after  EvalConst:                   {x}");

        MergePattern.visit_mut_expr(&mut x);
        trace!("after  MergePattern:                {x}");

        EvalConst.visit_mut_expr(&mut x);
        trace!("after  EvalConst:                   {x}");

        SimplifyByShortCircuit.visit_mut_expr(&mut x);
        trace!("after  SimplifyByShortCircuit:      {x}");

        EvalConst.visit_mut_expr(&mut x);
        trace!("after  EvalConst:                   {x}");
    }

    SimplifyTargetFamily.visit_mut_expr(&mut x);
    trace!("after  SimplifyTargetFamily:        {x}");

    SortByPriority.visit_mut_expr(&mut x);
    trace!("after  SortByPriority:              {x}");

    SortByValue.visit_mut_expr(&mut x);
    trace!("after  SortByValue:                 {x}");

    debug!("output:                             {x}");

    x
}

struct SortByPriority;

impl SortByPriority {
    fn get_priority(x: &Expr) -> Option<u32> {
        Some(match x {
            Expr::Not(_) => 103,
            Expr::Any(_) => 101,
            Expr::All(_) => 102,
            Expr::Var(Var(pred)) => match pred.key.as_str() {
                "target_family" => 1,
                "target_arch" => 2,
                "target_vendor" => 3,
                "target_os" => 4,
                "target_env" => 5,
                "target_pointer_width" => 6,
                _ => 0,
            },
            Expr::Const(_) => panic!(),
        })
    }
}

impl VisitMut<Pred> for SortByPriority {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Some(list) = expr.as_mut_expr_list() {
            list.sort_by(|lhs, rhs| {
                let Some(lhs) = Self::get_priority(lhs) else { return Equal };
                let Some(rhs) = Self::get_priority(rhs) else { return Equal };
                lhs.cmp(&rhs)
            })
        }

        walk_mut_expr(self, expr);
    }
}

struct SortByValue;

impl SortByValue {
    fn cmp_var(lhs: &Expr, rhs: &Expr) -> Ordering {
        let Expr::Var(Var(lhs)) = lhs else { return Equal };
        let Expr::Var(Var(rhs)) = rhs else { return Equal };

        let ok = Ord::cmp(lhs.key.as_str(), rhs.key.as_str());

        match (lhs.value.as_deref(), rhs.value.as_deref()) {
            (None, None) => ok,
            (Some(lv), Some(rv)) => ok.then_with(|| Ord::cmp(lv, rv)),
            (None, Some(_)) => Less,
            (Some(_), None) => Greater,
        }
    }

    fn cmp_not(lhs: &Expr, rhs: &Expr) -> Ordering {
        let Expr::Not(Not(lhs)) = lhs else { return Equal };
        let Expr::Not(Not(rhs)) = rhs else { return Equal };

        Self::cmp_var(lhs, rhs)
    }
}

impl VisitMut<Pred> for SortByValue {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Some(list) = expr.as_mut_expr_list() {
            list.sort_by(Self::cmp_var);
            list.sort_by(Self::cmp_not);
        }

        walk_mut_expr(self, expr);
    }
}

struct UnifyTargetFamily;

impl VisitMut<Pred> for UnifyTargetFamily {
    fn visit_mut_var(&mut self, Var(pred): &mut Var<Pred>) {
        if pred.value.is_none() && matches!(pred.key.as_str(), "unix" | "windows" | "wasm") {
            *pred = key_value("target_family", pred.key.as_str());
        }
    }
}

struct SimplifyTargetFamily;

impl VisitMut<Pred> for SimplifyTargetFamily {
    fn visit_mut_var(&mut self, Var(pred): &mut Var<Pred>) {
        if pred.key == "target_family" {
            if let Some(value) = pred.value.as_deref() {
                if matches!(value, "unix" | "windows" | "wasm") {
                    *pred = flag(value);
                }
            }
        }
    }
}

struct ImplyByKey;

impl ImplyByKey {
    const UNIQUE_VALUED_KEYS: &'static [&'static str] = &[
        "target_family",
        "target_arch",
        "target_vendor",
        "target_os",
        "target_env",
        "target_pointer_width",
    ];

    fn is_expr_any_pred(any: &[Expr], key: &str) -> bool {
        any.iter().all(|x| x.as_var().map_or(false, |Var(var)| var.key == key))
    }

    fn fix(pos_key: &str, pos_any_values: &[&str], expr: &mut Expr) {
        match expr {
            Expr::Any(Any(any)) => {
                any.iter_mut().for_each(|x| Self::fix(pos_key, pos_any_values, x));
            }
            Expr::All(All(all)) => {
                all.iter_mut().for_each(|x| Self::fix(pos_key, pos_any_values, x));
            }
            Expr::Not(Not(not)) => {
                Self::fix(pos_key, pos_any_values, not);
            }
            Expr::Var(Var(var)) => {
                if var.key == pos_key {
                    let var_value = var.value.as_deref().unwrap();
                    if pos_any_values.contains(&var_value) {
                        if pos_any_values.len() == 1 {
                            *expr = Expr::Const(true)
                        }
                    } else {
                        *expr = Expr::Const(false)
                    }
                }
            }
            Expr::Const(_) => {}
        }
    }
}

impl VisitMut<Pred> for ImplyByKey {
    fn visit_mut_all(&mut self, All(all): &mut All<Pred>) {
        walk_mut_expr_list(self, all);

        let mut i = 0;
        while i < all.len() {
            match &all[i] {
                Expr::Var(Var(pos)) => {
                    if Self::UNIQUE_VALUED_KEYS.contains(&pos.key.as_str()) {
                        assert!(pos.value.is_some());

                        let pos = pos.clone();
                        let pos_key = pos.key.as_str();
                        let pos_any_values = &[pos.value.as_deref().unwrap()];

                        for (_, x) in all.iter_mut().enumerate().filter(|&(j, _)| j != i) {
                            Self::fix(pos_key, pos_any_values, x);
                        }
                    }
                }
                Expr::Any(Any(any)) => {
                    if let Some(pos_key) = Self::UNIQUE_VALUED_KEYS.iter().find(|k| Self::is_expr_any_pred(any, k)) {
                        let any = any.clone();
                        let pos_any_values = map_collect_vec(&any, |x| x.as_var().unwrap().0.value.as_deref().unwrap());

                        for (_, x) in all.iter_mut().enumerate().filter(|&(j, _)| j != i) {
                            Self::fix(pos_key, &pos_any_values, x);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

struct SuppressTargetFamily;

impl SuppressTargetFamily {
    fn is_target_os_pred(x: &Expr) -> bool {
        match x {
            Expr::Var(Var(var)) => var.key == "target_os",
            _ => false,
        }
    }

    fn has_specified_target_os(x: &Expr) -> bool {
        if Self::is_target_os_pred(x) {
            return true;
        }

        if let Expr::Any(Any(any)) = x {
            return any.iter().all(Self::is_target_os_pred);
        }

        false
    }

    #[allow(clippy::match_like_matches_macro)]
    fn is_suppressed_target_family(pred: &Pred) -> bool {
        match (pred.key.as_str(), pred.value.as_deref()) {
            ("target_family", Some("unix")) => true,
            ("target_family", Some("windows")) => true,
            _ => false,
        }
    }
}

impl VisitMut<Pred> for SuppressTargetFamily {
    fn visit_mut_all(&mut self, All(all): &mut All<Pred>) {
        if all.iter().any(Self::has_specified_target_os) {
            all.remove_if(|x| match x {
                Expr::Var(Var(pred)) => Self::is_suppressed_target_family(pred),
                Expr::Not(Not(not)) => match &**not {
                    Expr::Var(Var(pred)) => Self::is_suppressed_target_family(pred),
                    _ => false,
                },
                _ => false,
            })
        }

        walk_mut_expr_list(self, all)
    }
}

struct MergePattern;

impl MergePattern {
    fn merge(any_list: &mut [Expr]) {
        let mut pattern_list = filter_map_collect_vec(any_list, |x| {
            if let Expr::All(All(all)) = x {
                if let [first, second] = all.as_mut_slice() {
                    if first.is_any() || first.is_var() {
                        return Some((first, second));
                    }
                }
            }
            None
        });

        if let [head, rest @ ..] = pattern_list.as_mut_slice() {
            let agg = match head.0 {
                Expr::Any(Any(any)) => any,
                Expr::Var(var) => {
                    *head.0 = expr(any((var.clone(),)));
                    head.0.as_mut_any().map(|x| &mut x.0).unwrap()
                }
                _ => panic!(),
            };

            for x in rest {
                let to_agg = if x.1 == head.1 {
                    &mut *x.0
                } else if x.0 == head.1 {
                    &mut *x.1
                } else {
                    continue;
                };

                match mem::replace(to_agg, Expr::Const(false)) {
                    Expr::Any(Any(any)) => agg.extend(any),
                    Expr::Var(var) => agg.push(expr(var.clone())),
                    other => *to_agg = other,
                }
            }

            if agg.len() == 1 {
                *head.0 = agg.pop().unwrap();
            }
        }
    }
}

impl VisitMut<Pred> for MergePattern {
    fn visit_mut_any(&mut self, Any(any_list): &mut Any<Pred>) {
        Self::merge(any_list);
        Self::merge(&mut any_list[1..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort() {
        let mut expr = expr(all((not(flag("unix")), flag("unix"))));
        SortByPriority.visit_mut_expr(&mut expr);
        assert_eq!(expr.to_string(), "all(unix, not(unix))");
    }

    #[test]
    fn imply() {
        {
            let mut expr = expr(all((target_os("linux"), not(target_os("emscripten")))));
            ImplyByKey.visit_mut_expr(&mut expr);
            assert_eq!(expr.to_string(), r#"all(target_os = "linux", not(false))"#)
        }
        {
            let mut expr = expr(all((
                any((target_os("ios"), target_os("macos"))),     //
                any((target_os("linux"), target_os("android"))), //
            )));
            ImplyByKey.visit_mut_expr(&mut expr);
            assert_eq!(expr.to_string(), r#"all(any(target_os = "ios", target_os = "macos"), any(false, false))"#);
        }
    }
}
