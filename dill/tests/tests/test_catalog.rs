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
#[should_panic]
fn test_add_impl_twice_panics() {
    let mut cat = CatalogBuilder::new();
    cat.add_value("foo".to_owned());
    cat.add_value("foo".to_owned());
}

#[test]
#[should_panic]
fn test_bind_with_no_impl_panics() {
    trait A {}

    #[component]
    struct AImpl;

    impl A for AImpl {}

    CatalogBuilder::new().bind::<dyn A, AImpl>();
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

#[test]
fn test_chained_catalog_binds() {
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

    let cat_earlier = CatalogBuilder::new()
        .add::<BImpl>()
        .bind::<dyn B, BImpl>()
        .build();

    let cat_later = CatalogBuilder::new_chained(&cat_earlier)
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .build();

    let inst_earlier_b = cat_earlier.get_one::<dyn B>().unwrap();
    assert_eq!(inst_earlier_b.test(), "bimpl");

    assert!(cat_earlier.get_one::<dyn A>().is_err());

    let inst_later_b = cat_later.get_one::<dyn B>().unwrap();
    assert_eq!(inst_later_b.test(), "bimpl");

    let inst_later_a = cat_later.get_one::<dyn A>().unwrap();
    assert_eq!(inst_later_a.test(), "aimpl::bimpl::foo");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_catalog_scope() {
    use std::assert_matches::assert_matches;

    let cat1 = Catalog::builder().add_value(1i32).build();

    let cat = cat1.clone();
    let proof = cat
        .scope(async move {
            // Get value from the current scope
            let l1_before = Catalog::current().get_one::<i32>().unwrap();
            assert_eq!(*l1_before.as_ref(), 1);

            // Nested scope with and additional registered value
            let cat2 = cat1.builder_chained().add_value(String::from("2")).build();
            let proof = cat2
                .scope(async move {
                    let l2 = Catalog::current().get_one::<String>().unwrap();
                    assert_eq!(l2.as_str(), "2");
                    l2
                })
                .await;

            // Check the scope was restored to cat1
            let l1_after = Catalog::current().get_one::<i32>().unwrap();
            assert_eq!(*l1_after.as_ref(), 1);
            assert_matches!(
                Catalog::current().get_one::<String>(),
                Err(InjectionError::Unregistered(_))
            );

            proof
        })
        .await;

    // This check is to ensure that all lambdas were actually executed and not
    // skipped
    assert_eq!(proof.as_str(), "2");
}
