use crate::ast::{All, Any, Expr, Not, Var};

#[allow(unused_variables)]
pub trait VisitMut<T> {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr)
    }

    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {
        walk_mut_expr_list(self, any);
    }

    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        walk_mut_expr_list(self, all);
    }

    fn visit_mut_not(&mut self, Not(not): &mut Not<T>) {
        walk_mut_not(self, not)
    }

    fn visit_mut_var(&mut self, var: &mut Var<T>) {}

    fn visit_mut_const(&mut self, b: &mut bool) {}
}

pub fn walk_mut_expr<V, T>(v: &mut V, expr: &mut Expr<T>)
where
    V: VisitMut<T> + ?Sized,
{
    match expr {
        Expr::Any(any) => {
            v.visit_mut_any(any);
        }
        Expr::All(all) => {
            v.visit_mut_all(all);
        }
        Expr::Not(not) => {
            v.visit_mut_not(not);
        }
        Expr::Var(var) => {
            v.visit_mut_var(var);
        }
        Expr::Const(b) => {
            v.visit_mut_const(b);
        }
    }
}

pub fn walk_mut_expr_list<V, T>(v: &mut V, list: &mut [Expr<T>])
where
    V: VisitMut<T> + ?Sized,
{
    for expr in list {
        v.visit_mut_expr(expr);
    }
}

pub fn walk_mut_not<V, T>(v: &mut V, not: &mut Expr<T>)
where
    V: VisitMut<T> + ?Sized,
{
    v.visit_mut_expr(not);
}
