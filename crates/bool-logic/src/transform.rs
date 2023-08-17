use crate::ast::{self, All, Any, Expr, Not, Var};
use crate::utils::*;
use crate::visit_mut::*;

use std::ops::Not as _;
use std::slice;

use replace_with::replace_with_or_abort as replace_with;
use rust_utils::default::default;
use rust_utils::iter::{filter_map_collect, map_collect_vec};
use rust_utils::slice::SliceExt;
use rust_utils::vec::VecExt;

fn unwrap_not<T>(expr: Expr<T>) -> Expr<T> {
    if let Expr::Not(Not(not)) = expr {
        *not
    } else {
        panic!()
    }
}

pub struct FlattenSingle;

impl<T> VisitMut<T> for FlattenSingle {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr);

        match expr {
            Expr::Any(Any(any)) => {
                if any.is_empty() {
                    *expr = Expr::Const(false);
                } else if any.len() == 1 {
                    *expr = any.pop().unwrap();
                }
            }
            Expr::All(All(all)) => {
                if all.is_empty() {
                    *expr = Expr::Const(true);
                } else if all.len() == 1 {
                    *expr = all.pop().unwrap();
                }
            }
            Expr::Not(Not(not_expr)) => {
                if not_expr.is_not() {
                    replace_with(expr, |expr| unwrap_not(unwrap_not(expr)))
                }
            }
            _ => {}
        };

        walk_mut_expr(self, expr)
    }
}

pub struct FlattenNestedList;

impl FlattenNestedList {
    fn flatten_any<T>(list: &mut Vec<Expr<T>>) {
        if list.iter().all(|x| x.is_any().not()) {
            return;
        }

        let mut ans: Vec<Expr<T>> = Vec::with_capacity(list.len());
        for expr in list.drain(..) {
            if let Expr::Any(Any(any)) = expr {
                ans.extend(any);
            } else {
                ans.push(expr);
            }
        }
        *list = ans;
    }

    fn flatten_all<T>(list: &mut Vec<Expr<T>>) {
        if list.iter().all(|x| x.is_all().not()) {
            return;
        }

        let mut ans: Vec<Expr<T>> = Vec::with_capacity(list.len());
        for expr in list.drain(..) {
            if let Expr::All(All(all)) = expr {
                ans.extend(all);
            } else {
                ans.push(expr);
            }
        }
        *list = ans;
    }
}

impl<T> VisitMut<T> for FlattenNestedList {
    fn visit_mut_any(&mut self, Any(list): &mut Any<T>) {
        Self::flatten_any(list);
        walk_mut_expr_list(self, list);
    }

    fn visit_mut_all(&mut self, All(list): &mut All<T>) {
        Self::flatten_all(list);
        walk_mut_expr_list(self, list);
    }
}

pub struct DedupList;

impl<T> VisitMut<T> for DedupList
where
    T: Eq,
{
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        if let Some(list) = expr.as_mut_expr_list() {
            let mut i = 0;
            while i < list.len() {
                let mut j = i + 1;
                while j < list.len() {
                    if list[i] == list[j] {
                        list.remove(j);
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
        walk_mut_expr(self, expr);
    }
}

pub struct EvalConst;

impl EvalConst {
    fn eval_any<T>(any: &mut Vec<Expr<T>>) -> Option<bool> {
        any.remove_if(|expr| expr.is_const_false());

        if any.is_empty() {
            return Some(false);
        }

        if any.iter().any(|expr| expr.is_const_true()) {
            return Some(true);
        }

        None
    }

    fn eval_all<T>(all: &mut Vec<Expr<T>>) -> Option<bool> {
        all.remove_if(|expr| expr.is_const_true());

        if all.is_empty() {
            return Some(true);
        }

        if all.iter().any(|expr| expr.is_const_false()) {
            return Some(false);
        }

        None
    }

    fn eval_not<T>(not: &Expr<T>) -> Option<bool> {
        if let Expr::Const(val) = not {
            return Some(val.not());
        }
        None
    }
}

impl<T> VisitMut<T> for EvalConst {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr);

        match expr {
            Expr::Any(Any(any)) => {
                if let Some(val) = Self::eval_any(any) {
                    *expr = Expr::Const(val);
                }
            }
            Expr::All(All(all)) => {
                if let Some(val) = Self::eval_all(all) {
                    *expr = Expr::Const(val);
                }
            }
            Expr::Not(Not(not)) => {
                if let Some(val) = Self::eval_not(not) {
                    *expr = Expr::Const(val);
                }
            }
            _ => {}
        }
    }
}

pub struct SimplifyNestedList;

impl SimplifyNestedList {
    fn contains_cross_same<T: Eq>(lhs: &[T], rhs: &[T]) -> bool {
        lhs.iter().any(|x| rhs.contains(x))
    }
}

impl<T> VisitMut<T> for SimplifyNestedList
where
    T: Eq,
{
    /// `any(x0, all(x0, x1), x2) => any(x0, x2)`
    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {
        let mut i = 0;
        while i < any.len() {
            if let Expr::All(All(all)) = &any[i] {
                if Self::contains_cross_same(all, any) {
                    any.remove(i);
                    continue;
                }
            }

            i += 1;
        }

        walk_mut_expr_list(self, any);
    }

    /// `all(x0, any(x0, x1), x2) => all(x0, x2)`
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        let mut i = 0;
        while i < all.len() {
            if let Expr::Any(Any(any)) = &all[i] {
                if Self::contains_cross_same(any, all) {
                    all.remove(i);
                    continue;
                }
            }

            i += 1;
        }

        walk_mut_expr_list(self, all);
    }
}

pub struct SimplifyAllNotAny;

impl SimplifyAllNotAny {
    /// Simplify `all(not(any(...)), any(...))`
    fn counteract<T: Eq>(neg: &[Expr<T>], pos: &mut Vec<Expr<T>>) {
        let mut i = 0;
        while i < pos.len() {
            if neg.contains(&pos[i]) {
                pos.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

impl<T> VisitMut<T> for SimplifyAllNotAny
where
    T: Eq,
{
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        if let [Expr::Not(Not(not)), Expr::Any(Any(pos))] = all.as_mut_slice() {
            let neg = match not.as_mut_any() {
                Some(Any(neg)) => neg,
                None => slice::from_mut(&mut **not),
            };
            Self::counteract(neg, pos);
        } else if let [Expr::Any(Any(pos)), Expr::Not(Not(not))] = all.as_mut_slice() {
            let neg = match not.as_mut_any() {
                Some(Any(neg)) => neg,
                None => slice::from_mut(&mut **not),
            };
            Self::counteract(neg, pos);
        }

        walk_mut_expr_list(self, all);
    }
}

pub struct FlattenByDeMorgan;

impl<T> VisitMut<T> for FlattenByDeMorgan {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        if let Expr::Not(Not(not)) = expr {
            match &mut **not {
                Expr::Any(Any(any)) => {
                    let list = map_collect_vec(any.drain(..), |expr| ast::expr(ast::not(expr)));
                    *expr = ast::expr(ast::all(list));
                }
                Expr::All(All(all)) => {
                    let list = map_collect_vec(all.drain(..), |expr| ast::expr(ast::not(expr)));
                    *expr = ast::expr(ast::any(list));
                }
                _ => {}
            }
        }

        walk_mut_expr(self, expr)
    }
}

pub struct MergeAllOfNotAny;

impl MergeAllOfNotAny {
    fn as_mut_not_any<T>(expr: &mut Expr<T>) -> Option<&mut Vec<Expr<T>>> {
        expr.as_mut_not_any().map(|x| &mut x.0)
    }

    fn unwrap_expr_not_var<T>(expr: Expr<T>) -> Var<T> {
        if let Expr::Not(Not(not)) = expr {
            if let Expr::Var(var) = *not {
                return var;
            }
        }
        panic!()
    }
}

impl<T> VisitMut<T> for MergeAllOfNotAny {
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        let mut not_any_list: Vec<_> = filter_map_collect(&mut *all, Self::as_mut_not_any);

        if let [first, rest @ ..] = not_any_list.as_mut_slice() {
            if rest.is_empty().not() {
                rest.iter_mut().for_each(|x| first.append(x));
                all.remove_if(|x| x.is_empty_not_any())
            }

            {
                let not_var_list: Vec<_> = drain_filter(all, |x| x.is_expr_not_var()).collect();
                let not_any = all.iter_mut().find_map(Self::as_mut_not_any).unwrap();

                for not_var in not_var_list {
                    let var = Self::unwrap_expr_not_var(not_var);
                    not_any.push(ast::expr(var));
                }
            }
        }
    }
}

pub struct MergeAllOfAny;

impl MergeAllOfAny {
    fn is_subset_of<T: Eq>(lhs: &[Expr<T>], rhs: &[Expr<T>]) -> bool {
        lhs.iter().all(|x| rhs.contains(x))
    }
}

impl<T: Eq> VisitMut<T> for MergeAllOfAny {
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        walk_mut_expr_list(self, all);

        let mut any_list: Vec<_> = filter_map_collect(&mut *all, |x| Expr::as_mut_any(x).map(|x| &mut x.0));

        for i in 0..any_list.len() {
            for j in 0..any_list.len() {
                if let Some((lhs, rhs)) = any_list.get2_mut(i, j) {
                    if Self::is_subset_of(lhs, rhs) {
                        rhs.clear();
                        rhs.push(Expr::Const(true));
                    }
                }
            }
        }
    }
}

pub struct SimplifyByShortCircuit;

impl SimplifyByShortCircuit {
    fn find_vars<T: Eq + Clone>(list: &mut [Expr<T>], marker: bool) -> Vec<Var<T>> {
        let mut ans: Vec<Var<T>> = default();
        for x in list {
            if let Expr::Var(var) = x {
                if ans.contains(var) {
                    *x = Expr::Const(marker);
                } else {
                    ans.push(var.clone())
                }
            }
        }
        ans
    }

    fn replace_vars<T: Eq>(x: &mut Expr<T>, vars: &[Var<T>], marker: bool) {
        match x {
            Expr::Any(Any(any)) => any.iter_mut().for_each(|x| Self::replace_vars(x, vars, marker)),
            Expr::All(All(all)) => all.iter_mut().for_each(|x| Self::replace_vars(x, vars, marker)),
            Expr::Not(Not(not)) => Self::replace_vars(not, vars, marker),
            Expr::Var(var) => {
                if vars.contains(var) {
                    *x = Expr::Const(marker);
                }
            }
            Expr::Const(_) => {}
        }
    }
}

impl<T: Eq + Clone> VisitMut<T> for SimplifyByShortCircuit {
    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {
        let marker = false;
        let vars = Self::find_vars(any, marker);
        for x in any.iter_mut().filter(|x| x.is_var().not()) {
            Self::replace_vars(x, &vars, marker);
        }

        walk_mut_expr_list(self, any)
    }

    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        let marker = true;
        let vars = Self::find_vars(all, marker);
        for x in all.iter_mut().filter(|x| x.is_var().not()) {
            Self::replace_vars(x, &vars, marker);
        }

        walk_mut_expr_list(self, all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast::*;

    #[test]
    fn eval_const() {
        let mut cfg: Expr<u32> = expr(not(not(any(()))));
        EvalConst.visit_mut_expr(&mut cfg);
        assert_eq!(cfg.to_string(), "false");
    }
}
