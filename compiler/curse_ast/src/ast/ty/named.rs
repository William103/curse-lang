use crate::ast::{tok, Iter, Path, Type};
use crate::ast_struct;
use curse_span::{HasSpan, Span};

ast_struct! {
    /// A named type, e.g. `std::vec::Vec T`
    #[derive(Clone, Debug)]
    pub struct NamedType {
        pub path: Path,
        pub generic_args: Option<GenericArgs>,
    }
}

#[derive(Clone, Debug)]
pub enum GenericArgs {
    /// Example: `Vec I32`
    Single(Type),
    /// Example: `Result (I32 * Error)`
    CartesianProduct(tok::LParen, Vec<(Type, tok::Star)>, Type, tok::RParen),
}

impl GenericArgs {
    pub fn iter_args(&self) -> Iter<'_, Type, tok::Star> {
        let (slice, last) = match self {
            GenericArgs::Single(last) => (&[] as _, Some(last)),
            GenericArgs::CartesianProduct(_, vec, last, _) => (vec.as_slice(), Some(last)),
        };

        Iter::new(slice.iter(), last)
    }
}

impl HasSpan for NamedType {
    fn start(&self) -> u32 {
        self.path.start()
    }

    fn end(&self) -> u32 {
        if let Some(generic_args) = self.generic_args.as_ref() {
            generic_args.end()
        } else {
            self.path.end()
        }
    }
}

impl HasSpan for GenericArgs {
    fn start(&self) -> u32 {
        match self {
            GenericArgs::Single(ty) => ty.start(),
            GenericArgs::CartesianProduct(lparen, ..) => lparen.start(),
        }
    }

    fn end(&self) -> u32 {
        match self {
            GenericArgs::Single(ty) => ty.end(),
            GenericArgs::CartesianProduct(.., rparen) => rparen.end(),
        }
    }

    fn span(&self) -> Span {
        match self {
            GenericArgs::Single(ty) => ty.span(),
            GenericArgs::CartesianProduct(lparen, .., rparen) => Span {
                start: lparen.start(),
                end: rparen.end(),
            },
        }
    }
}
