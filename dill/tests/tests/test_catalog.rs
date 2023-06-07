use std::sync::Arc;

use dill::*;

#[test]
fn test_add_value() {
    let mut cat = CatalogBuilder::new();
    cat.add_value("foo".to_owned());
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");

    let val2 = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ptr(), val2.as_ptr());
}

#[test]
fn test_add_value_lazy() {
    let mut cat = CatalogBuilder::new();
    cat.add_value_lazy(|| "foo".to_owned());
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");

    let val2 = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ptr(), val2.as_ptr());
}

#[test]
fn test_add_builder_arc() {
    let mut cat = CatalogBuilder::new();
    cat.add_builder(Arc::new("foo".to_owned()));
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}

#[test]
fn test_add_builder_fn() {
    let mut cat = CatalogBuilder::new();
    cat.add_builder(|| Arc::new("foo".to_owned()));
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}

#[test]
fn test_self_injection() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl1 {
        b: Arc<dyn B>,
    }

    #[component]
    impl AImpl1 {
        fn new(catalog: &Catalog) -> Self {
            Self {
                b: catalog.get_one().unwrap(),
            }
        }
    }

    impl A for AImpl1 {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b.test())
        }
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    struct BImpl {
        c: Arc<C>,
    }

    #[component]
    impl BImpl {
        fn new(catalog: Catalog) -> Self {
            Self {
                c: catalog.get_one().unwrap(),
            }
        }
    }

    impl B for BImpl {
        fn test(&self) -> String {
            format!("bimpl::{}", self.c.test())
        }
    }

    #[component]
    struct C;

    impl C {
        fn test(&self) -> String {
            "c".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl1>()
        .bind::<dyn A, AImpl1>()
        .add::<BImpl>()
        .bind::<dyn B, BImpl>()
        .add::<C>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl::c");
}
