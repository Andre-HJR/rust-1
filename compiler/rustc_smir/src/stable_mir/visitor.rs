use std::ops::ControlFlow;

use crate::rustc_internal::Opaque;

use super::ty::{
    Allocation, Binder, Const, ConstDef, ExistentialPredicate, FnSig, GenericArgKind, GenericArgs,
    Promoted, RigidTy, TermKind, Ty, UnevaluatedConst,
};

pub trait Visitor: Sized {
    type Break;
    fn visit_ty(&mut self, ty: &Ty) -> ControlFlow<Self::Break> {
        ty.super_visit(self)
    }
    fn visit_const(&mut self, c: &Const) -> ControlFlow<Self::Break> {
        c.super_visit(self)
    }
}

pub trait Visitable {
    fn visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.super_visit(visitor)
    }
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break>;
}

impl Visitable for Ty {
    fn visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        visitor.visit_ty(self)
    }
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self.kind() {
            super::ty::TyKind::RigidTy(ty) => ty.visit(visitor),
            super::ty::TyKind::Alias(_, alias) => alias.args.visit(visitor),
            super::ty::TyKind::Param(_) => todo!(),
            super::ty::TyKind::Bound(_, _) => todo!(),
        }
    }
}

impl Visitable for Const {
    fn visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        visitor.visit_const(self)
    }
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match &self.literal {
            super::ty::ConstantKind::Allocated(alloc) => alloc.visit(visitor),
            super::ty::ConstantKind::Unevaluated(uv) => uv.visit(visitor),
            super::ty::ConstantKind::ParamCt(param) => param.visit(visitor),
        }
    }
}

impl Visitable for Opaque {
    fn super_visit<V: Visitor>(&self, _visitor: &mut V) -> ControlFlow<V::Break> {
        ControlFlow::Continue(())
    }
}

impl Visitable for Allocation {
    fn super_visit<V: Visitor>(&self, _visitor: &mut V) -> ControlFlow<V::Break> {
        ControlFlow::Continue(())
    }
}

impl Visitable for UnevaluatedConst {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        let UnevaluatedConst { ty, def, args, promoted } = self;
        ty.visit(visitor)?;
        def.visit(visitor)?;
        args.visit(visitor)?;
        promoted.visit(visitor)
    }
}

impl Visitable for ConstDef {
    fn super_visit<V: Visitor>(&self, _visitor: &mut V) -> ControlFlow<V::Break> {
        ControlFlow::Continue(())
    }
}

impl<T: Visitable> Visitable for Option<T> {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            Some(val) => val.visit(visitor),
            None => ControlFlow::Continue(()),
        }
    }
}

impl Visitable for Promoted {
    fn super_visit<V: Visitor>(&self, _visitor: &mut V) -> ControlFlow<V::Break> {
        ControlFlow::Continue(())
    }
}

impl Visitable for GenericArgs {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.0.visit(visitor)
    }
}

impl Visitable for GenericArgKind {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            GenericArgKind::Lifetime(lt) => lt.visit(visitor),
            GenericArgKind::Type(t) => t.visit(visitor),
            GenericArgKind::Const(c) => c.visit(visitor),
        }
    }
}

impl Visitable for RigidTy {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            RigidTy::Bool
            | RigidTy::Char
            | RigidTy::Int(_)
            | RigidTy::Uint(_)
            | RigidTy::Float(_)
            | RigidTy::Never
            | RigidTy::Foreign(_)
            | RigidTy::Str => ControlFlow::Continue(()),
            RigidTy::Array(t, c) => {
                t.visit(visitor)?;
                c.visit(visitor)
            }
            RigidTy::Slice(inner) => inner.visit(visitor),
            RigidTy::RawPtr(ty, _) => ty.visit(visitor),
            RigidTy::Ref(_, ty, _) => ty.visit(visitor),
            RigidTy::FnDef(_, args) => args.visit(visitor),
            RigidTy::FnPtr(sig) => sig.visit(visitor),
            RigidTy::Closure(_, args) => args.visit(visitor),
            RigidTy::Generator(_, args, _) => args.visit(visitor),
            RigidTy::Dynamic(pred, r, _) => {
                pred.visit(visitor)?;
                r.visit(visitor)
            }
            RigidTy::Tuple(fields) => fields.visit(visitor),
            RigidTy::Adt(_, args) => args.visit(visitor),
        }
    }
}

impl<T: Visitable> Visitable for Vec<T> {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        for arg in self {
            arg.visit(visitor)?;
        }
        ControlFlow::Continue(())
    }
}

impl<T: Visitable> Visitable for Binder<T> {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.value.visit(visitor)
    }
}

impl Visitable for ExistentialPredicate {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            ExistentialPredicate::Trait(tr) => tr.generic_args.visit(visitor),
            ExistentialPredicate::Projection(p) => {
                p.term.visit(visitor)?;
                p.generic_args.visit(visitor)
            }
            ExistentialPredicate::AutoTrait(_) => ControlFlow::Continue(()),
        }
    }
}

impl Visitable for TermKind {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            TermKind::Type(t) => t.visit(visitor),
            TermKind::Const(c) => c.visit(visitor),
        }
    }
}

impl Visitable for FnSig {
    fn super_visit<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.inputs_and_output.visit(visitor)
    }
}
