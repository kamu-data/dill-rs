use std::assert_matches::assert_matches;
use std::sync::Arc;

use dill::*;

#[test]
fn test_one_of_unregistered() {
    let cat = CatalogBuilder::new().build();

    let res = cat.get::<OneOf<i32>>();
    assert_matches!(res, Err(e) if e == InjectionError::unregistered::<i32>());
}

#[test]
fn test_one_of_same_type() {
    #[component]
    struct X;

    impl X {
        fn test(&self) -> String {
            "hello".to_owned()
        }
    }

    let mut cat = CatalogBuilder::new();
    cat.add::<X>();
    let cat = cat.build();

    let inst = cat.get::<OneOf<X>>().unwrap();
    assert_eq!(inst.test(), "hello");
}

#[test]
fn test_one_of_by_interface() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl;

    impl A for AImpl {
        fn test(&self) -> String {
            "aimpl".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl");
}

#[test]
fn test_one_of_by_multiple_interfaces() {
    trait A: Send + Sync {
        fn foo(&self) -> String;
    }

    trait B: Send + Sync {
        fn bar(&self) -> String;
    }

    #[component]
    struct ABImpl;

    impl A for ABImpl {
        fn foo(&self) -> String {
            "abimpl".to_owned()
        }
    }

    impl B for ABImpl {
        fn bar(&self) -> String {
            "abimpl".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add::<ABImpl>()
        .bind::<dyn A, ABImpl>()
        .bind::<dyn B, ABImpl>()
        .build();

    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.foo(), "abimpl");
    let inst = cat.get::<OneOf<dyn B>>().unwrap();
    assert_eq!(inst.bar(), "abimpl");
}

#[test]
fn test_one_of_with_dependency() {
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
    assert_eq!(inst.test(), "aimpl::bimpl");
}

#[test]
fn test_one_of_with_dependency_missing() {
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

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .build();

    let res = cat.get::<OneOf<dyn A>>();
    assert_matches!(res.err(), Some(e) if e == InjectionError::unregistered::<dyn B>());
}

#[test]
fn test_all_of() {
    trait A {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl1;

    impl A for AImpl1 {
        fn test(&self) -> String {
            "aimpl1".to_owned()
        }
    }

    #[component]
    struct AImpl2;

    impl A for AImpl2 {
        fn test(&self) -> String {
            "aimpl2".to_owned()
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl1>()
        .add::<AImpl2>()
        .bind::<dyn A, AImpl1>()
        .bind::<dyn A, AImpl2>()
        .build();

    let instances = cat.get::<AllOf<dyn A>>().unwrap();
    let mut vals: Vec<_> = instances.iter().map(|i| i.test()).collect();
    vals.sort();

    assert_eq!(vals, vec!["aimpl1", "aimpl2"]);
}

#[test]
fn test_add_value() {
    let mut cat = CatalogBuilder::new();
    cat.add_value("foo".to_owned());
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}

#[test]
fn test_add_factory() {
    let mut cat = CatalogBuilder::new();
    cat.add_factory(|| "foo".to_owned());
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
