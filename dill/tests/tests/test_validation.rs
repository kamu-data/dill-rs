use std::assert_matches::assert_matches;
use std::sync::Arc;

use dill::*;

#[test]
fn test_validate_static_graph() {
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

    // Success
    b.add::<BImpl>();
    b.bind::<dyn B, BImpl>();

    assert_matches!(b.validate(), Ok(()));

    // Catalog still works
    let cat = b.build();
    cat.get_one::<dyn A>().unwrap();
}
