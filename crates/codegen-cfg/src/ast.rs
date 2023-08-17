use std::fmt;

pub use bool_logic::ast::All;
pub use bool_logic::ast::Any;
pub use bool_logic::ast::Not;
pub use bool_logic::ast::Var;

pub type Expr = bool_logic::ast::Expr<Pred>;

pub fn expr(x: impl Into<Expr>) -> Expr {
    x.into()
}

pub fn any(x: impl Into<Any<Pred>>) -> Any<Pred> {
    x.into()
}

pub fn all(x: impl Into<All<Pred>>) -> All<Pred> {
    x.into()
}

pub fn not(x: impl Into<Not<Pred>>) -> Not<Pred> {
    x.into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pred {
    pub key: String,
    pub value: Option<String>,
}

impl From<Pred> for Expr {
    fn from(x: Pred) -> Self {
        Expr::Var(Var(x))
    }
}

pub fn flag(s: impl Into<String>) -> Pred {
    Pred {
        key: s.into(),
        value: None,
    }
}

pub fn key_value(s: impl Into<String>, v: impl Into<String>) -> Pred {
    Pred {
        key: s.into(),
        value: Some(v.into()),
    }
}

pub fn target_family(s: impl Into<String>) -> Pred {
    key_value("target_family", s)
}

pub fn target_vendor(s: impl Into<String>) -> Pred {
    key_value("target_vendor", s)
}

pub fn target_arch(s: impl Into<String>) -> Pred {
    key_value("target_arch", s)
}

pub fn target_os(s: impl Into<String>) -> Pred {
    key_value("target_os", s)
}

pub fn target_env(s: impl Into<String>) -> Pred {
    key_value("target_env", s)
}

pub fn target_pointer_width(s: impl Into<String>) -> Pred {
    key_value("target_pointer_width", s)
}

impl fmt::Display for Pred {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = self.key.as_str();
        match &self.value {
            Some(value) => write!(f, "{key} = {value:?}"),
            None => write!(f, "{key}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_string() {
        {
            let cfg = expr(target_family("unix"));
            let expected = r#"target_family = "unix""#;
            assert_eq!(cfg.to_string(), expected);
        }
        {
            let cfg = expr(any((target_os("linux"), target_os("android"))));
            let expected = r#"any(target_os = "linux", target_os = "android")"#;
            assert_eq!(cfg.to_string(), expected);
        }
    }
}
