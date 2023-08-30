use std::sync::Arc;

#[test]
fn test_transient() {
    use dill::*;

    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    // #[scope(Transient)]  Expecting default
    struct AImpl {
        // Needed for compiler not to optimize type out
        name: String,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.name)
        }
    }

    let cat = CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .add_value("foo".to_owned())
        .build();

    let inst1 = cat.get::<OneOf<dyn A>>().unwrap();
    let inst2 = cat.get::<OneOf<dyn A>>().unwrap();

    assert_ne!(
        inst1.as_ref() as *const dyn A,
        inst2.as_ref() as *const dyn A
    );

    assert_eq!(inst1.test(), "aimpl::foo");
    assert_eq!(inst2.test(), "aimpl::foo");
}

#[test]
fn test_singleton() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[dill::component]
    #[dill::scope(dill::Singleton)]
    struct AImpl {
        // Needed for compiler not to optimize type out
        name: String,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.name)
        }
    }

    let cat = dill::CatalogBuilder::new()
        .add::<AImpl>()
        .bind::<dyn A, AImpl>()
        .add_value("foo".to_owned())
        .build();

    let inst1 = cat.get::<dill::OneOf<dyn A>>().unwrap();
    let inst2 = cat.get::<dill::OneOf<dyn A>>().unwrap();

    assert_eq!(
        inst1.as_ref() as *const dyn A,
        inst2.as_ref() as *const dyn A
    );

    assert_eq!(inst1.test(), "aimpl::foo");
    assert_eq!(inst2.test(), "aimpl::foo");
}

#[test]
fn test_chained_singleton() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        // Needed for compiler not to optimize type out
        name: String,
        b: Arc<dyn B>,
    }

    #[dill::component]
    #[dill::scope(dill::Singleton)]
    impl AImpl {
        fn new(name: String, b: Arc<dyn B>) -> Self {
            Self { name, b }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.name, self.b.test())
        }
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    struct BImpl {
        last_name: String,
    }

    #[dill::component]
    #[dill::scope(dill::Singleton)]
    impl BImpl {
        pub fn new(last_name: String) -> Self {
            Self { last_name }
        }
    }

    impl B for BImpl {
        fn test(&self) -> String {
            format!("bimpl::{}", self.last_name)
        }
    }

    let cat_earlier = dill::CatalogBuilder::new()
        .add_builder(dill::builder_for::<AImpl>().with_name("test".to_string()))
        .bind::<dyn A, AImpl>()
        .build();

    let cat_later = dill::CatalogBuilder::new_chained(&cat_earlier)
        .add_value(BImpl::new("unique".to_string()))
        .bind::<dyn B, BImpl>()
        .build();

    // Nothing implements B in earlier catalog, so neither A, nor B can be created
    assert!(cat_earlier.get::<dill::OneOf<dyn A>>().is_err());
    assert!(cat_earlier.get::<dill::OneOf<dyn B>>().is_err());

    let inst_a_1 = cat_later.get::<dill::OneOf<dyn A>>().unwrap();
    let inst_a_2 = cat_later.get::<dill::OneOf<dyn A>>().unwrap();

    assert_eq!(
        inst_a_1.as_ref() as *const dyn A,
        inst_a_2.as_ref() as *const dyn A
    );

    let inst_b_1 = cat_later.get::<dill::OneOf<dyn B>>().unwrap();
    let inst_b_2 = cat_later.get::<dill::OneOf<dyn B>>().unwrap();

    assert_eq!(
        inst_b_1.as_ref() as *const dyn B,
        inst_b_2.as_ref() as *const dyn B
    );

    assert_eq!(
        inst_b_1.as_ref() as *const dyn B,
        inst_b_2.as_ref() as *const dyn B
    );

    assert_eq!(inst_a_1.test(), "aimpl::test::bimpl::unique");
    assert_eq!(inst_a_2.test(), "aimpl::test::bimpl::unique");

    assert_eq!(inst_b_1.test(), "bimpl::unique");
    assert_eq!(inst_b_2.test(), "bimpl::unique");
}
