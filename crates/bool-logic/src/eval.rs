use crate::ast::{All, Any, Expr, Not, Var};

pub fn eval_with<T, F>(expr: &Expr<T>, f: &F) -> bool
where
    F: for<'v> Fn(&'v T) -> bool,
{
    match expr {
        Expr::Any(Any(list)) => list.iter().any(|e| eval_with(e, f)),
        Expr::All(All(list)) => list.iter().all(|e| eval_with(e, f)),
        Expr::Not(Not(not)) => !eval_with(not, f),
        Expr::Var(Var(var)) => f(var),
        Expr::Const(b) => *b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast::*;

    #[test]
    fn flatten_cfg_if() {
        let (x0, x1, x2) = (var(0), var(1), var(2));
        let src = expr(all((not(any((x0, x1))), x2)));
        let dst = expr(all((not(x0), not(x1), x2)));

        for i in 0..8 {
            let assign = |x: &_| ((i >> x) & 1) == 1;
            let lhs = eval_with(&src, &assign);
            let rhs = eval_with(&dst, &assign);
            assert_eq!(lhs, rhs, "i = {i}");
        }
    }
}
