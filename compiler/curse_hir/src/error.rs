use crate::{Type, Var, spanned::S};
use curse_ast::{self as ast, Span};
use miette::{Diagnostic, SourceSpan, NamedSource};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
#[error("A lowering error occurred.")]
pub struct SourceErrors<'hir> {
    #[source_code]
    pub code: NamedSource,

    #[related]
    pub errors: Vec<LowerError<'hir>>,
}

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum LowerError<'hir> {
    #[error("Cannot unify types: {ty1} and {ty2}")]
    #[diagnostic(help("Use types that can be unified"))]
    Unify {
        #[label("First type")]
        ty1_span: (usize, usize),
        ty1: Type<'hir>,

        #[label("Second type")]
        ty2_span: (usize, usize),
        ty2: Type<'hir>,
    },

    #[error("Infinite recursive type")]
    #[diagnostic(help("Don't make the type depend on itself."))]
    CyclicType {
        #[label("Type that could not be assigned")]
        var_span: (usize, usize),
        var: Var,

        #[label("Type that was attempted to be assigned to")]
        ty_span: (usize, usize),
        ty: Type<'hir>,
    },

    #[error("Identifier not found: `{literal}`")]
    #[diagnostic(help("Use an identifier that is in scope"))]
    IdentNotFound {
        #[label("This identifier here")]
        span: SourceSpan,
        literal: String,
    },

    #[error("Cannot parse integer {literal}")]
    #[diagnostic(help("Use a smaller integer"))]
    ParseInt {
        #[label("This number here")]
        span: SourceSpan,
        literal: String,
    },
}

impl From<&ast::tok::Integer<'_>> for LowerError<'_> {
    fn from(value: &ast::tok::Integer<'_>) -> Self {
        LowerError::ParseInt {
            span: value.span().into(),
            literal: value.literal.to_string(),
        }
    }
}

impl From<&ast::tok::Ident<'_>> for LowerError<'_> {
    fn from(value: &ast::tok::Ident<'_>) -> Self {
        LowerError::IdentNotFound {
            span: value.span().into(),
            literal: value.literal.to_string(),
        }
    }
}

impl<'hir> LowerError<'hir> {
    pub fn unify(t1: S<Type<'hir>>, t2: S<Type<'hir>>) -> Self {
        LowerError::Unify {
            ty1_span: t1.span(),
            ty1: *t1.value(),
            ty2_span: t2.span(),
            ty2: *t2.value(),
        }
    }
}
