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
        #[allow(dead_code)]
        fn test(&self) -> String;
    }

    #[allow(dead_code)]
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
        #[allow(dead_code)]
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
        maybe_val: Option<i32>,
    }

    let cat = CatalogBuilder::new().add::<BImpl>().build();

    let inst = cat.get_one::<BImpl>().unwrap();
    assert_matches!(inst.maybe_a, None);
    assert_matches!(inst.maybe_val, None);

    let cat = CatalogBuilder::new()
        .add::<BImpl>()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .add_value(42i32)
        .build();

    let inst = cat.get_one::<BImpl>().unwrap();
    assert_matches!(inst.maybe_a, Some(_));
    assert_matches!(inst.maybe_val, Some(42));
}

#[test]
fn test_lazy_simple() {
    #[component]
    #[derive(Debug)]
    struct A;

    impl A {
        fn test(&self) -> String {
            "A".into()
        }
    }

    let cat = Catalog::builder().add::<A>().build();

    let lazy_a = cat.get::<dill::specs::Lazy<OneOf<A>>>().unwrap();
    let a = lazy_a.get().unwrap();
    assert_eq!(a.test(), "A");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_lazy_scoped() {
    #[component]
    #[derive(Debug)]
    struct A;

    impl A {
        fn test(&self) -> String {
            "A".into()
        }
    }

    let cat = Catalog::builder().build();

    let lazy_a = cat.get::<dill::specs::Lazy<OneOf<A>>>().unwrap();
    assert_matches!(lazy_a.get(), Err(InjectionError::Unregistered(_)));

    let cat2 = cat.builder_chained().add::<A>().build();

    let test = cat2
        .scope(async move {
            let a = lazy_a.get().unwrap();
            a.test()
        })
        .await;

    assert_eq!(test, "A");
}

#[test]
fn test_lazy_derive() {
    #[component]
    #[derive(Debug)]
    struct A;

    impl A {
        fn test(&self) -> String {
            "A".into()
        }
    }

    #[component]
    #[derive(Debug)]
    struct B {
        lazy_a: dill::Lazy<Arc<A>>,
    }

    impl B {
        fn test(&self) -> String {
            let a = self.lazy_a.get().unwrap();
            a.test()
        }
    }

    let cat = Catalog::builder().add::<A>().add::<B>().build();

    let b = cat.get_one::<B>().unwrap();
    assert_eq!(b.test(), "A");
}
