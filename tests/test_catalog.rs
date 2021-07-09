#![feature(assert_matches)]

use std::sync::Arc;

use dill::*;

#[test]
fn test_one_of_unregistered() {
    let cat = Catalog::new();
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

    let mut cat = Catalog::new();
    cat.add::<X>();
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

    let mut cat = Catalog::new();
    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();
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

    let mut cat = Catalog::new();
    cat.add::<ABImpl>();
    cat.bind::<dyn A, ABImpl>().unwrap();
    cat.bind::<dyn B, ABImpl>().unwrap();
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

    let mut cat = Catalog::new();

    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();

    cat.add::<BImpl>();
    cat.bind::<dyn B, BImpl>().unwrap();

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

    let mut cat = Catalog::new();

    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();

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

    let mut cat = Catalog::new();
    cat.add::<AImpl1>();
    cat.add::<AImpl2>();
    cat.bind::<dyn A, AImpl1>().unwrap();
    cat.bind::<dyn A, AImpl2>().unwrap();

    let instances = cat.get::<AllOf<dyn A>>().unwrap();
    let mut vals: Vec<_> = instances.iter().map(|i| i.test()).collect();
    vals.sort();

    assert_eq!(vals, vec!["aimpl1", "aimpl2"]);
}

#[test]
fn test_add_value() {
    let mut cat = Catalog::new();
    cat.add_value("foo".to_owned());

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}

#[test]
fn test_add_factory() {
    let mut cat = Catalog::new();
    cat.add_factory(|| "foo".to_owned());

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}
