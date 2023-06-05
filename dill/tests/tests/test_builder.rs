use std::any::TypeId;
use std::sync::Arc;

use dill::*;

#[test]
fn test_type_info() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl;

    impl A for AImpl {
        fn test(&self) -> String {
            "foo".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .build();

    let builders: Vec<_> = cat.builders_for::<dyn A>().collect();
    assert_eq!(builders.len(), 1);

    let builder = builders.into_iter().next().unwrap();
    assert_eq!(builder.instance_type_id(), TypeId::of::<AImpl>());
    assert_eq!(
        builder.instance_type_name(),
        "unit::tests::test_builder::test_type_info::AImpl"
    );
}

#[test]
fn test_with_args_by_value() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl {
        host: String,
        port: i32,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.host, self.port)
        }
    }

    let cat = CatalogBuilder::new()
        .add_builder(
            builder_for::<AImpl>()
                .with_host("foo".to_owned())
                .with_port(8080),
        )
        .bind::<dyn A, AImpl>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo::8080");

    let cat = CatalogBuilder::new()
        .add_builder(
            builder_for::<AImpl>()
                .with_host_fn(|_| Ok("bar".to_owned()))
                .with_port_fn(|_| Ok(8080)),
        )
        .bind::<dyn A, AImpl>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bar::8080");
}

#[test]
fn test_with_args_by_ref() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl {
        b: Arc<dyn B>,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b.test())
        }
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    struct BImpl1;

    impl B for BImpl1 {
        fn test(&self) -> String {
            "bimpl1".to_owned()
        }
    }

    #[component]
    struct BImpl2;

    impl B for BImpl2 {
        fn test(&self) -> String {
            "bimpl2".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add_builder(builder_for::<AImpl>().with_b(Arc::new(BImpl1)))
        .bind::<dyn A, AImpl>()
        .add::<BImpl2>()
        .bind::<dyn B, BImpl2>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl1");

    let cat = CatalogBuilder::new()
        .add_builder(builder_for::<AImpl>().with_b_fn(|_| Ok(Arc::new(BImpl1))))
        .bind::<dyn A, AImpl>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl1");
}

#[test]
fn test_new_ctor() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        b: Arc<dyn B>,
        suffix: String,
    }

    #[component]
    impl AImpl {
        pub fn new(bee: Arc<dyn B>) -> Self {
            Self {
                b: bee,
                suffix: "foo".to_owned(),
            }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.b.test(), self.suffix)
        }
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct BImpl;

    impl B for BImpl {
        fn test(&self) -> String {
            "bimpl".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .add::<BImpl>()
        .bind::<dyn B, BImpl>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl::foo");
}

#[test]
fn test_new_ctor_cloned() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        b: B,
    }

    #[component]
    impl AImpl {
        pub fn new(bee: B) -> Self {
            Self { b: bee }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b.0)
        }
    }

    #[derive(Clone)]
    struct B(String);

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .add_value(B("foo".to_owned()))
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo");
}

#[test]
fn test_new_ctor_by_ref() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        b: String,
    }

    #[component]
    impl AImpl {
        pub fn new(bee: &B) -> Self {
            Self { b: bee.0.clone() }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b)
        }
    }

    struct B(String);

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .add_value(B("foo".to_owned()))
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo");
}

/*#[test]
fn test_generic_type_from_struct() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl<T> {
        t: T,
    }

    impl<T> A for AImpl<T>
    where
        T: Send + Sync,
        T: Display,
    {
        fn test(&self) -> String {
            format!("aimpl::{}", self.t)
        }
    }

    struct B(String);

    impl Display for B {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl<B>>()
        .bind::<dyn A, AImpl<B>>()
        .add_value(B("foo".to_owned()))
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo");
}*/

/*#[test]
fn test_generic_type_from_impl() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl<T> {
        b: String,
        _p: PhantomData<T>,
    }

    #[component]
    impl<T> AImpl<T> {
        pub fn new(bee: &B) -> Self {
            Self {
                b: bee.0.clone(),
                _p: PhantomData,
            }
        }
    }

    impl<T> A for AImpl<T> {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.b, std::any::type_name::<T>())
        }
    }

    struct B(String);

    let cat = CatalogBuilder::new()
        .add::<AImpl<u8>>()
        .bind::<dyn A, AImpl<u8>>()
        .add_value(B("foo".to_owned()))
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo::u8");
}*/
