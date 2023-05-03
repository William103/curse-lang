#![allow(dead_code)]
use ast::Span;
use curse_ast as ast;
use smallvec::{smallvec, SmallVec};
use std::{
    collections::HashMap,
    iter,
    ops::{Index, IndexMut},
};
use typed_arena::Arena;

mod equations;
pub use equations::{Edge, Equations, Node};

mod expr;
pub use expr::*;

pub mod dot;

mod error;
pub use error::*;

pub mod usefulness;

mod types;
pub use types::*;

mod lowering;
pub use lowering::*;

#[derive(Clone, Debug)]
pub struct Polytype<'hir> {
    pub typevars: SmallVec<[Var; 4]>,
    pub ty: Type<'hir>,
}

impl<'hir> Polytype<'hir> {
    pub fn new(ty: Type<'hir>) -> Self {
        Polytype {
            typevars: SmallVec::new(),
            ty,
        }
    }
}

pub struct Hir<'hir, 'input> {
    pub type_fns: &'hir Arena<TypeFunction<'hir>>,
    pub appls: &'hir Arena<ExprAppl<'hir, 'input>>,
    pub arms: &'hir Arena<ExprArm<'hir, 'input>>,
    pub exprs: &'hir Arena<Expr<'hir, 'input>>,
    pub types: &'hir Arena<Type<'hir>>,
    pub pats: &'hir Arena<Pat<'hir, 'input>>,
    pub typevars: Vec<Typevar<'hir>>,
    pub equations: Equations<'hir>,
}

impl<'hir, 'input> Hir<'hir, 'input> {
    pub fn new_typevar(&mut self) -> Var {
        let var = Var(self.typevars.len());
        self.typevars.push(Typevar::Unbound);
        var
    }

    /// Get a global environment with the type signatures of `in` and `print`
    /// already loaded in.
    ///
    /// `in`: `x (x () -> y) -> y`
    ///
    /// and
    ///
    /// `print`: `x () -> ()`
    pub fn default_globals(&mut self) -> impl Iterator<Item = (&'static str, Polytype<'hir>)> {
        let dummy = (0, 0);
        [
            ("in", {
                let x = self.new_typevar();
                let x_type = Type {
                    kind: TypeKind::Var(x),
                    span: dummy,
                };
                let y = self.new_typevar();
                let y_type = Type {
                    kind: TypeKind::Var(y),
                    span: dummy,
                };

                Polytype {
                    typevars: smallvec![x, y],
                    ty: Type {
                        kind: TypeKind::Function(self.type_fns.alloc(TypeFunction {
                            lhs: x_type,
                            rhs: Type {
                                kind: TypeKind::Function(self.type_fns.alloc(TypeFunction {
                                    lhs: x_type,
                                    rhs: Type {
                                        kind: TypeKind::unit(),
                                        span: dummy,
                                    },
                                    output: y_type,
                                })),
                                span: dummy,
                            },
                            output: y_type,
                        })),
                        span: dummy,
                    },
                }
            }),
            ("print", {
                let x = self.new_typevar();
                let x_type = Type {
                    kind: TypeKind::Var(x),
                    span: dummy,
                };

                Polytype {
                    typevars: smallvec![x],
                    ty: Type {
                        kind: TypeKind::Function(self.type_fns.alloc(TypeFunction {
                            lhs: x_type,
                            rhs: Type {
                                kind: TypeKind::unit(),
                                span: dummy,
                            },
                            output: Type {
                                kind: TypeKind::unit(),
                                span: dummy,
                            },
                        })),
                        span: dummy,
                    },
                }
            }),
        ]
        .into_iter()
    }

    pub fn monomorphize(&mut self, polytype: &Polytype<'hir>) -> Type<'hir> {
        // Takes a polymorphic type and replaces all instances of generics
        // with a fixed, unbound type.
        // For example, id: T -> T is a polymorphic type, so it goes through
        // and replaces both `T`s with an unbound type variable like `a0`,
        // which is then bound later on.
        fn replace_unbound_typevars<'hir>(
            tbl: &HashMap<Var, TypeKind<'hir>>,
            hir: &mut Hir<'hir, '_>,
            ty: Type<'hir>,
        ) -> Type<'hir> {
            match ty.kind {
                TypeKind::Var(var) => Type {
                    kind: tbl[&var],
                    ..ty
                },
                TypeKind::Function(fun) => Type {
                    kind: TypeKind::Function(hir.type_fns.alloc(TypeFunction {
                        lhs: replace_unbound_typevars(tbl, hir, fun.lhs),
                        rhs: replace_unbound_typevars(tbl, hir, fun.rhs),
                        output: replace_unbound_typevars(tbl, hir, fun.output),
                    })),
                    ..ty
                },
                TypeKind::Tuple(types) => {
                    let replaced_types = hir
                        .types
                        .alloc_extend(iter::repeat_with(Type::dummy).take(types.len()));
                    for (i, ty) in types.iter().enumerate() {
                        replaced_types[i] = replace_unbound_typevars(tbl, hir, *ty);
                    }

                    Type {
                        kind: TypeKind::Tuple(replaced_types),
                        ..ty
                    }
                }
                _ => ty,
            }
        }

        let tvs_to_replace = polytype
            .typevars
            .iter()
            .map(|tv| (*tv, TypeKind::Var(self.new_typevar())))
            .collect();

        replace_unbound_typevars(&tvs_to_replace, self, polytype.ty)
    }

    /// Convert an [`ast::Type<'_, 'input>`] annotation into an HIR [`Type<'hir>`].
    pub fn type_from_ast(
        &mut self,
        ty: &ast::Type<'_, 'input>,
        map: &HashMap<&str, Type<'hir>>,
    ) -> Type<'hir> {
        match ty {
            ast::Type::Named(named) => match named.name.literal {
                "i32" => Type {
                    kind: TypeKind::I32,
                    span: named.span(),
                },
                "bool" => Type {
                    kind: TypeKind::Bool,
                    span: named.span(),
                },
                other => map.get(other).copied().expect("type not found"),
            },
            ast::Type::Tuple(tuple) => {
                let types = self
                    .types
                    .alloc_extend(iter::repeat_with(Type::dummy).take(tuple.len()));

                for (i, ty) in tuple.iter_elements().enumerate() {
                    types[i] = self.type_from_ast(ty, map);
                }

                Type {
                    kind: TypeKind::Tuple(types),
                    span: tuple.span(),
                }
            }
            ast::Type::Function(fun) => {
                let lhs = self.type_from_ast(fun.lhs, map);
                let rhs = self.type_from_ast(fun.rhs, map);
                let output = self.type_from_ast(fun.ret, map);

                Type {
                    kind: TypeKind::Function(self.type_fns.alloc(TypeFunction {
                        lhs,
                        rhs,
                        output,
                    })),
                    span: fun.span(),
                }
            }
        }
    }

    fn occurs(&self, var: Var, ty: &Type<'_>) -> bool {
        match ty.kind {
            TypeKind::Var(typevar) => {
                if let Some(t) = self[typevar].binding() {
                    self.occurs(var, t)
                } else {
                    var == typevar
                }
            }
            TypeKind::Function(fun) => {
                self.occurs(var, &fun.lhs)
                    || self.occurs(var, &fun.rhs)
                    || self.occurs(var, &fun.output)
            }
            _ => false,
        }
    }

    fn check_equivalence(&self, var: Var, ty: Type<'_>) -> bool {
        if let Type {
            kind: TypeKind::Var(typevar),
            ..
        } = ty
        {
            if let Some(t) = self[typevar].binding() {
                self.check_equivalence(var, *t)
            } else {
                var == typevar
            }
        } else {
            false
        }
    }
}

impl<'hir> Index<Var> for Hir<'hir, '_> {
    type Output = Typevar<'hir>;

    fn index(&self, index: Var) -> &Self::Output {
        &self.typevars[index.0 as usize]
    }
}

impl<'hir> IndexMut<Var> for Hir<'hir, '_> {
    fn index_mut(&mut self, index: Var) -> &mut Self::Output {
        &mut self.typevars[index.0 as usize]
    }
}
