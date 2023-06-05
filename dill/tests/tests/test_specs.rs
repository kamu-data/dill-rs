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
fn test_all_of_derive() {
    trait A: Send + Sync {}

    #[component]
    struct AImpl1;
    impl A for AImpl1 {}

    #[component]
    struct AImpl2;
    impl A for AImpl2 {}

    #[component]
    struct BImpl {
        vec_of_a: Vec<Arc<dyn A>>,
    }

    let cat = CatalogBuilder::new().add::<BImpl>().build();

    assert_eq!(cat.get_one::<BImpl>().unwrap().vec_of_a.len(), 0);

    let cat = CatalogBuilder::new()
        .add::<BImpl>()
        .add::<AImpl1>()
        .bind::<dyn A, AImpl1>()
        .add::<AImpl2>()
        .bind::<dyn A, AImpl2>()
        .build();

    assert_eq!(cat.get_one::<BImpl>().unwrap().vec_of_a.len(), 2);
}

#[test]
fn test_maybe() {
    trait A: std::fmt::Debug + Send + Sync {}
    trait B: std::fmt::Debug + Send + Sync {}

    #[component]
    #[derive(Debug)]
    struct AImpl;
    impl A for AImpl {}

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .build();

    assert_matches!(cat.get::<Maybe<OneOf<dyn B>>>().unwrap(), None);
    assert_matches!(cat.get::<Maybe<OneOf<dyn A>>>().unwrap(), Some(_));
    assert_matches!(cat.get::<Maybe<AllOf<dyn A>>>().unwrap(), Some(v) if v.len() == 1);
}

#[test]
fn test_maybe_derive() {
    trait A: std::fmt::Debug + Send + Sync {}

    #[component]
    #[derive(Debug)]
    struct AImpl;
    impl A for AImpl {}

    #[component]
    #[derive(Debug)]
    struct BImpl {
        maybe_a: Option<Arc<dyn A>>,
    }

    let cat = CatalogBuilder::new().add::<BImpl>().build();

    assert_matches!(cat.get_one::<BImpl>().unwrap().maybe_a, None);

    let cat = CatalogBuilder::new()
        .add::<BImpl>()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .build();

    assert_matches!(cat.get_one::<BImpl>().unwrap().maybe_a, Some(_));
}
