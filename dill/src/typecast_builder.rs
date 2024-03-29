use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

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

/////////////////////////////////////////////////////////////////////////////////////////

/// Takes a dynamic `Builder` and casts the instance to desired interface
pub struct TypecastBuilder<'a, Iface>
where
    Iface: 'static + ?Sized,
{
    builder: &'a dyn Builder,
    caster: &'a TypeCaster<Iface>,
}

impl<'a, Iface> Builder for TypecastBuilder<'a, Iface>
where
    Iface: 'static + ?Sized,
{
    fn instance_type_id(&self) -> TypeId {
        self.builder.instance_type_id()
    }

    fn instance_type_name(&self) -> &'static str {
        self.builder.instance_type_name()
    }

    fn interfaces(&self) -> Vec<InterfaceDesc> {
        self.builder.interfaces()
    }

    fn get(&self, cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        self.builder.get(cat)
    }

    fn check(&self, cat: &Catalog) -> Result<(), ValidationError> {
        self.builder.check(cat)
    }
}

impl<'a, Iface> TypecastBuilder<'a, Iface>
where
    Iface: 'static + ?Sized,
{
    fn new(builder: &'a dyn Builder, caster: &'a TypeCaster<Iface>) -> Self {
        Self { builder, caster }
    }

    pub fn get(&self, cat: &Catalog) -> Result<Arc<Iface>, InjectionError> {
        let inst = self.builder.get(cat)?;
        Ok((self.caster.cast_arc)(inst))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct TypeCaster<Into: ?Sized> {
    pub cast_arc: fn(Arc<dyn Any + Send + Sync>) -> Arc<Into>,
}

pub(crate) type AnyTypeCaster = dyn Any + Send + Sync;

/////////////////////////////////////////////////////////////////////////////////////////

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
        if let Some(bindings) = self.bindings {
            let prev_pos = self.pos;
            self.pos += 1;
            bindings.get(prev_pos).map(|b| {
                // SAFETY: the TypeID key of the `bindings` map is guaranteed to match the
                // `Iface` type
                let caster: &TypeCaster<Iface> = b.caster.downcast_ref().unwrap();
                TypecastBuilder::new(b.builder.as_ref(), caster)
            })
        } else {
            None
        }
    }
}
