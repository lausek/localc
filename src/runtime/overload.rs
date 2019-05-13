use super::*;

use std::cmp::Ordering;

// `Overload` is a function signature where `Expr` is either a constant or an identifier.

#[derive(Clone, Debug, PartialEq)]
pub struct Overload(Vec<Expr>);

impl Overload {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Expr> {
        self.0.iter()
    }

    // tests if the arguments supplied satisfy the `Overload`s constraints
    pub fn accepts(&self, args: Vec<Expr>) -> bool {
        // equal if both are empty
        if self.0.is_empty() && args.is_empty() {
            return true;
        }
        // size must be equal
        if self.count() != args.len() {
            return false;
        }

        for (p, a) in self.0.iter().zip(args.iter()) {
            match p {
                Expr::Value(_) if p == a => {}
                Expr::Ref(_) => {}
                _ => return false,
            }
        }

        true
    }
}

impl<T> From<Vec<T>> for Overload
where
    T: Into<Expr>,
{
    fn from(from: Vec<T>) -> Self {
        let exprs = from.into_iter().map(|f| f.into()).collect::<Vec<_>>();
        Overload(exprs)
    }
}

impl std::cmp::Eq for Overload {}

impl std::cmp::Ord for Overload {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::cmp::PartialOrd for Overload {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.count(), other.count()) {
            (s, o) if s == 0 && o == 0 => Some(Ordering::Equal),
            (s, o) if s < o => Some(Ordering::Less),
            (s, o) if s > o => Some(Ordering::Greater),
            _ => {
                for (s, o) in self.0.iter().zip(other.iter()) {
                    let result = match (s, o) {
                        (Expr::Value(s), Expr::Value(o)) => s.partial_cmp(&o),
                        (Expr::Ref(s), Expr::Ref(o)) => s.partial_cmp(&o),
                        (Expr::Value(_), _) => Some(Ordering::Less),
                        (Expr::Ref(_), _) => Some(Ordering::Greater),
                        // only `Value` and `Ref` are allowed in overload
                        _ => unreachable!(),
                    };
                    if result != Some(Ordering::Equal) {
                        return result;
                    }
                }
                Some(Ordering::Equal)
            }
        }
    }
}
