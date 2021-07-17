use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::{PhantomData, Unsize},
    sync::Arc,
};

use multimap::MultiMap;

use crate::{
    Builder, BuilderLike, DependencySpec, Factory, InjectionError, OneOf, Prebuilt, TypedBuilder,
};

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

    fn get(&self, cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        self.builder.get(cat)
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

struct TypeCaster<Into: ?Sized> {
    cast_arc: fn(Arc<dyn Any + Send + Sync>) -> Arc<Into>,
}

type AnyTypeCaster = dyn Any + Send + Sync;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct IfaceTypeId(TypeId);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct ImplTypeId(TypeId);

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: CoW?
#[derive(Clone)]
pub struct Catalog {
    builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    bindings: MultiMap<IfaceTypeId, Binding>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
            bindings: MultiMap::new(),
        }
    }

    pub fn add<Bld: BuilderLike>(&mut self) {
        Bld::register(self);
    }

    pub fn add_builder<Bld, Impl>(&mut self, builder: Bld)
    where
        Impl: 'static + Send + Sync,
        Bld: TypedBuilder<Impl> + 'static,
    {
        let builder = Arc::new(builder);
        self.builders
            .insert(ImplTypeId(TypeId::of::<Impl>()), builder.clone());

        self.bindings.insert(
            IfaceTypeId(TypeId::of::<Impl>()),
            Binding::new(
                Arc::new(TypeCaster::<Impl> {
                    // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the `Impl` instances
                    cast_arc: |v| v.downcast().unwrap(),
                }),
                builder,
            ),
        );
    }

    pub fn add_factory<Fct, Impl>(&mut self, factory: Fct)
    where
        Fct: 'static + Fn() -> Impl + Send + Sync,
        Impl: 'static + Send + Sync,
    {
        self.add_builder(Factory::new(factory))
    }

    pub fn add_value<'a, Impl>(&'a mut self, value: Impl)
    where
        Impl: 'static + Send + Sync,
    {
        self.add_builder(Prebuilt::new(value))
    }

    // TODO: WTF is Unsize
    pub fn bind<Iface, Impl>(&mut self) -> Result<(), InjectionError>
    where
        Iface: 'static + ?Sized,
        Impl: 'static + Send + Sync + Unsize<Iface>,
    {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());
        let impl_type = ImplTypeId(TypeId::of::<Impl>());

        let builder = self.builders.get(&impl_type);
        if builder.is_none() {
            return Err(InjectionError::unregistered::<Iface>());
        }

        self.bindings.insert(
            iface_type,
            Binding::new(
                Arc::new(TypeCaster::<Iface> {
                    cast_arc: |v| {
                        // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the `Impl` instances
                        let s: Arc<Impl> = v.downcast().unwrap();
                        let t: Arc<Iface> = s;
                        t
                    },
                }),
                builder.unwrap().clone(),
            ),
        );

        Ok(())
    }

    pub fn builders_for<'a, Iface: 'static + ?Sized>(
        &'a self,
    ) -> impl Iterator<Item = TypecastBuilder<'a, Iface>> {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());

        let bindings = self.bindings.get_vec(&&iface_type);
        TypecastBuilderIterator::new(bindings)
    }

    pub fn get<Spec: DependencySpec>(&self) -> Result<Spec::ReturnType, InjectionError> {
        Spec::get(self)
    }

    pub fn get_one<Iface>(&self) -> Result<Arc<Iface>, InjectionError>
    where
        Iface: 'static + ?Sized + Send + Sync,
    {
        OneOf::get(self)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub fn builder_for<B: BuilderLike>() -> B::Builder {
    B::builder()
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct Binding {
    caster: Arc<AnyTypeCaster>,
    builder: Arc<dyn Builder>,
}

impl Binding {
    fn new(caster: Arc<AnyTypeCaster>, builder: Arc<dyn Builder>) -> Self {
        Self { caster, builder }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

struct TypecastBuilderIterator<'a, Iface: 'static + ?Sized> {
    bindings: Option<&'a Vec<Binding>>,
    pos: usize,
    _dummy: PhantomData<Iface>,
}

impl<'a, Iface: 'static + ?Sized> TypecastBuilderIterator<'a, Iface> {
    fn new(bindings: Option<&'a Vec<Binding>>) -> Self {
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
                // SAFETY: the TypeID key of the `bindings` map is guaranteed to match the `Iface` type
                let caster: &TypeCaster<Iface> = b.caster.downcast_ref().unwrap();
                TypecastBuilder::new(b.builder.as_ref(), caster)
            })
        } else {
            None
        }
    }
}
