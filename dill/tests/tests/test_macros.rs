#[test]
fn test_macro_without_use() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[dill::component]
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

    let inst1 = cat.get_one::<dyn A>().unwrap();
    let inst2 = cat.get_one::<dyn A>().unwrap();

    assert_ne!(
        inst1.as_ref() as *const dyn A,
        inst2.as_ref() as *const dyn A
    );

    assert_eq!(inst1.test(), "aimpl::foo");
    assert_eq!(inst2.test(), "aimpl::foo");
}
