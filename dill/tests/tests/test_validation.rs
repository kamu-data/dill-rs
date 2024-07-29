use std::assert_matches::assert_matches;
use std::sync::Arc;

use dill::*;

#[test]
fn test_validate_static_graph() {
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

    #[component]
    struct BImpl;
    impl B for BImpl {
        fn test(&self) -> String {
            "bimpl".to_owned()
        }
    }

    // Unresolved
    let mut b = CatalogBuilder::new();
    b.add::<AImpl>();
    b.bind::<dyn A, AImpl>();

    let res = b.validate();
    assert_matches!(
        &res,
        Err(ValidationError { errors }) if errors.len() == 1
    );

    let err = &res.err().unwrap().errors[0];
    assert_matches!(
        err,
        InjectionError::Unregistered(u)
        if u.type_name == "dyn unit::tests::test_validation::test_validate_static_graph::B"
    );

    // Consider B is registered dynamically
    let res = b.validate().ignore::<dyn B>();
    assert_matches!(res, Ok(()));

    // Success
    b.add::<BImpl>();
    b.bind::<dyn B, BImpl>();

    assert_matches!(b.validate(), Ok(()));

    // Catalog still works
    let cat = b.build();
    cat.get_one::<dyn A>().unwrap();
}

#[test]
fn test_validate_ingores_bound_fields() {
    trait A: Send + Sync {}

    #[allow(dead_code)]
    #[component]
    struct AImpl {
        foo: i32,
    }
    impl A for AImpl {}

    let mut b = CatalogBuilder::new();
    b.add_builder(AImpl::builder().with_foo(10));
    b.bind::<dyn A, AImpl>();

    b.validate().unwrap();
}

#[test]
fn test_validate_catalog_inject() {
    trait A: Send + Sync {}

    #[allow(dead_code)]
    struct AImpl {
        catalog: Catalog,
    }

    #[component]
    impl AImpl {
        pub fn new(catalog: Catalog) -> Self {
            Self { catalog }
        }
    }
    impl A for AImpl {}

    let mut b = CatalogBuilder::new();
    b.add::<AImpl>();
    b.bind::<dyn A, AImpl>();

    b.validate().unwrap();
}
