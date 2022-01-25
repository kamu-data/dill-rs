use dill::*;

#[test]
fn test_transient() {
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

    #[component]
    #[scope(Singleton)]
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

    assert_eq!(
        inst1.as_ref() as *const dyn A,
        inst2.as_ref() as *const dyn A
    );

    assert_eq!(inst1.test(), "aimpl::foo");
    assert_eq!(inst2.test(), "aimpl::foo");
}
