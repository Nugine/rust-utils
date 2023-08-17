use crate::ast::*;

use std::backtrace::Backtrace;
use std::ops::Not as _;
use std::panic::Location;

use rust_utils::default::default;

#[derive(Debug)]
pub struct Error<'p> {
    pub input: &'p str,
    pub location: &'static Location<'static>,
    pub backtrace: Backtrace,
}

pub type Result<'p, T, E = Error<'p>> = std::result::Result<T, E>;

macro_rules! ensure {
    ($s:expr, $cond:expr) => {
        if !$cond {
            return Err(Error {
                input: $s,
                location: Location::caller(),
                backtrace: Backtrace::capture(),
            });
        }
    };
}

pub fn parse(mut s: &str) -> Result<Expr> {
    let s = &mut s;
    skip_space0(s);
    let expr = parse_expr(s)?;
    skip_space0(s);
    ensure!(s, s.is_empty());
    Ok(expr)
}

fn take_while1<'p>(s: &mut &'p str, f: impl Fn(char) -> bool) -> Result<'p, &'p str> {
    let end = s.char_indices().find_map(|(i, c)| f(c).not().then_some(i)).unwrap_or(s.len());

    ensure!(s, end > 0);

    let (ans, rest) = s.split_at(end);
    *s = rest;
    Ok(ans)
}

fn consume_tag<'p>(s: &mut &'p str, tag: &str) -> Result<'p, &'p str> {
    ensure!(s, s.starts_with(tag));
    let (ans, rest) = s.split_at(tag.len());
    *s = rest;
    Ok(ans)
}

fn skip_space0(s: &mut &str) {
    *s = s.trim_start();
}

fn skip_tag<'p>(s: &mut &'p str, tag: &str) -> Option<&'p str> {
    if s.starts_with(tag) {
        let (ans, rest) = s.split_at(tag.len());
        *s = rest;
        Some(ans)
    } else {
        None
    }
}

// https://doc.rust-lang.org/reference/conditional-compilation.html

fn parse_expr<'p>(s: &mut &'p str) -> Result<'p, Expr> {
    if s.starts_with("any") {
        Ok(expr(parse_any(s)?))
    } else if s.starts_with("all") {
        Ok(expr(parse_all(s)?))
    } else if s.starts_with("not") {
        Ok(expr(parse_not(s)?))
    } else {
        Ok(expr(parse_pred(s)?))
    }
}

fn parse_pred<'p>(s: &mut &'p str) -> Result<'p, Pred> {
    let key = parse_identifier(s)?;

    let has_value = s.trim_start().starts_with('=');

    let value = if has_value {
        skip_space0(s);
        skip_tag(s, "=");
        skip_space0(s);
        Some(parse_string_literal(s)?)
    } else {
        None
    };

    Ok(Pred {
        key: key.into(),
        value: value.map(Into::into),
    })
}

// TODO: unicode identifier
fn parse_identifier<'p>(s: &mut &'p str) -> Result<'p, &'p str> {
    ensure!(s, s.starts_with(|c| matches!(c, 'a'..='z'|'A'..='Z'|'_')));
    take_while1(s, |c| matches!(c, 'a'..='z'|'A'..='Z'|'0'..='9'|'_'))
}

// TODO: escaped string
fn parse_string_literal<'p>(s: &mut &'p str) -> Result<'p, &'p str> {
    consume_tag(s, "\"")?;

    let ans = take_while1(s, |c| c != '"')?;
    assert!(ans.contains('\\').not());

    consume_tag(s, "\"")?;

    Ok(ans)
}

fn parse_any<'p>(s: &mut &'p str) -> Result<'p, Any<Pred>> {
    consume_tag(s, "any")?;
    skip_space0(s);
    consume_tag(s, "(")?;
    let list = parse_expr_list(s)?;
    skip_space0(s);
    consume_tag(s, ")")?;
    Ok(any(list))
}

fn parse_all<'p>(s: &mut &'p str) -> Result<'p, All<Pred>> {
    consume_tag(s, "all")?;
    skip_space0(s);
    consume_tag(s, "(")?;
    let list = parse_expr_list(s)?;
    skip_space0(s);
    consume_tag(s, ")")?;
    Ok(all(list))
}

fn parse_not<'p>(s: &mut &'p str) -> Result<'p, Not<Pred>> {
    consume_tag(s, "not")?;
    skip_space0(s);
    consume_tag(s, "(")?;
    let expr = parse_expr(s)?;
    skip_space0(s);
    consume_tag(s, ")")?;
    Ok(not(expr))
}

fn parse_expr_list<'p>(s: &mut &'p str) -> Result<'p, Vec<Expr>> {
    let mut ans: Vec<Expr> = default();

    while s.starts_with(')').not() {
        skip_space0(s);
        ans.push(parse_expr(s)?);
        skip_space0(s);
        skip_tag(s, ",");
    }

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_string() {
        let input = r#"  all(not(any(target_os = "linux", target_os = "macos")), all(), any(unix))  "#;
        let expr = parse(input).unwrap();
        assert_eq!(expr.to_string(), input.trim());
    }
}
