use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::injection_context::InjectionContext;
use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct Binding {
    pub caster: Arc<AnyTypeCaster>,
    pub builder: Arc<dyn Builder>,
}

impl Binding {
    pub(crate) fn new(caster: Arc<AnyTypeCaster>, builder: Arc<dyn Builder>) -> Self {
        Self { caster, builder }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Takes a dynamic `Builder` and casts the instance to desired interface
pub struct TypecastBuilder<'a, Iface>
where
    Iface: 'static + ?Sized,
{
    pub(crate) builder: &'a Arc<dyn Builder>,
    pub(crate) caster: &'a TypeCaster<Iface>,
}

impl<Iface> Builder for TypecastBuilder<'_, Iface>
where
    Iface: 'static + ?Sized,
{
    fn instance_type(&self) -> TypeInfo {
        self.builder.instance_type()
    }

    fn scope_type(&self) -> TypeInfo {
        self.builder.scope_type()
    }

    fn interfaces(&self, clb: &mut dyn FnMut(&TypeInfo) -> bool) {
        self.builder.interfaces(clb);
    }

    fn dependencies(&self, clb: &mut dyn FnMut(&DependencyInfo) -> bool) {
        self.builder.dependencies(clb);
    }

    fn metadata<'b, 'c>(&'b self, clb: &'c mut dyn FnMut(&'b dyn Any) -> bool) {
        self.builder.metadata(clb)
    }

    fn get_any(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        self.builder.get_any(cat, ctx)
    }

    fn check(&self, cat: &Catalog, ctx: &InjectionContext) -> Result<(), ValidationError> {
        self.builder.check(cat, ctx)
    }
}

impl<'a, Iface> TypecastBuilder<'a, Iface>
where
    Iface: 'static + ?Sized,
{
    fn new(builder: &'a Arc<dyn Builder>, caster: &'a TypeCaster<Iface>) -> Self {
        Self { builder, caster }
    }

    pub fn get(&self, cat: &Catalog) -> Result<Arc<Iface>, InjectionError> {
        self.get_with_context(cat, &InjectionContext::new_root())
    }

    pub fn get_with_context(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<Iface>, InjectionError> {
        let inst = self.builder.get_any(cat, ctx)?;
        Ok((self.caster.cast_arc)(inst))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct TypeCaster<Into: ?Sized> {
    pub cast_arc: fn(Arc<dyn Any + Send + Sync>) -> Arc<Into>,
}

pub(crate) type AnyTypeCaster = dyn Any + Send + Sync;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct TypecastBuilderIterator<'a, Iface: 'static + ?Sized> {
    bindings: Option<&'a Vec<Binding>>,
    pos: usize,
    _dummy: PhantomData<Iface>,
}

impl<'a, Iface: 'static + ?Sized> TypecastBuilderIterator<'a, Iface> {
    pub(crate) fn new(bindings: Option<&'a Vec<Binding>>) -> Self {
        Self {
            bindings,
            pos: 0,
            _dummy: PhantomData,
        }
    }
}

impl<'a, Iface: 'static + ?Sized> Iterator for TypecastBuilderIterator<'a, Iface> {
    type Item = TypecastBuilder<'a, Iface>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bindings) = self.bindings
            && self.pos < bindings.len()
        {
            let b = &bindings[self.pos];
            self.pos += 1;

            // SAFETY: the TypeID key of the `bindings` map is guaranteed to match the
            // `Iface` type
            let caster: &TypeCaster<Iface> = b.caster.downcast_ref().unwrap();
            return Some(TypecastBuilder::new(&b.builder, caster));
        }
        None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct TypecastPredicateBuilderIterator<'a, Iface: 'static + ?Sized, Pred>
where
    Pred: Fn(&dyn Builder) -> bool,
{
    bindings: Option<&'a Vec<Binding>>,
    pred: Pred,
    pos: usize,
    _dummy: PhantomData<Iface>,
}

impl<'a, Iface: 'static + ?Sized, Pred> TypecastPredicateBuilderIterator<'a, Iface, Pred>
where
    Pred: Fn(&dyn Builder) -> bool,
{
    pub(crate) fn new(bindings: Option<&'a Vec<Binding>>, pred: Pred) -> Self {
        Self {
            bindings,
            pred,
            pos: 0,
            _dummy: PhantomData,
        }
    }
}

impl<'a, Iface: 'static + ?Sized, Pred> Iterator
    for TypecastPredicateBuilderIterator<'a, Iface, Pred>
where
    Pred: Fn(&dyn Builder) -> bool,
{
    type Item = TypecastBuilder<'a, Iface>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bindings) = self.bindings {
            while self.pos < bindings.len() {
                let b = &bindings[self.pos];
                self.pos += 1;

                if (self.pred)(b.builder.as_ref()) {
                    // SAFETY: the TypeID key of the `bindings` map is guaranteed to match the
                    // `Iface` type
                    let caster: &TypeCaster<Iface> = b.caster.downcast_ref().unwrap();
                    return Some(TypecastBuilder::new(&b.builder, caster));
                }
            }
        }
        None
    }
}
