use std::assert_matches::assert_matches;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Transient
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Singleton
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_chained_singleton() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        // Needed for compiler not to optimize type out
        name: String,
        b: Option<Arc<dyn B>>,
    }

    #[dill::component]
    #[dill::scope(dill::Singleton)]
    impl AImpl {
        fn new(name: String, b: Option<Arc<dyn B>>) -> Self {
            Self { name, b }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!(
                "aimpl::{}::{}",
                self.name,
                match &self.b {
                    Some(b) => b.test(),
                    None => "no-b".to_string(),
                }
            )
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

    use dill::Component;

    let cat_earlier = dill::CatalogBuilder::new()
        .add_builder(AImpl::builder().with_name("test".to_string()))
        .bind::<dyn A, AImpl>()
        .build();

    let cat_later = dill::CatalogBuilder::new_chained(&cat_earlier)
        .add_value(BImpl::new("unique".to_string()))
        .bind::<dyn B, BImpl>()
        .build();

    let inst_a_1 = cat_earlier.get::<dill::OneOf<dyn A>>().unwrap();
    let inst_a_2 = cat_earlier.get::<dill::OneOf<dyn A>>().unwrap();
    assert_eq!(
        inst_a_1.as_ref() as *const dyn A,
        inst_a_2.as_ref() as *const dyn A
    );

    let inst_a_3 = cat_later.get::<dill::OneOf<dyn A>>().unwrap();
    let inst_a_4 = cat_later.get::<dill::OneOf<dyn A>>().unwrap();

    assert_eq!(
        inst_a_3.as_ref() as *const dyn A,
        inst_a_4.as_ref() as *const dyn A
    );
    assert_eq!(
        inst_a_2.as_ref() as *const dyn A,
        inst_a_3.as_ref() as *const dyn A
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

    assert_eq!(inst_a_1.test(), "aimpl::test::no-b");
    assert_eq!(inst_a_1.test(), "aimpl::test::no-b");
    assert_eq!(inst_a_3.test(), "aimpl::test::no-b");
    assert_eq!(inst_a_4.test(), "aimpl::test::no-b");

    assert_eq!(inst_b_1.test(), "bimpl::unique");
    assert_eq!(inst_b_2.test(), "bimpl::unique");
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Transaction
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_scope_transaction() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        pub name: Arc<std::sync::Mutex<String>>,
        pub b: Option<Arc<dyn B>>,
    }

    #[dill::component]
    #[dill::interface(dyn A)]
    impl AImpl {
        fn new(b: Option<Arc<dyn B>>) -> Self {
            Self {
                name: Default::default(),
                b,
            }
        }

        pub fn set_name(&self, name: impl Into<String>) {
            *self.name.lock().unwrap() = name.into();
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!(
                "{}::{}",
                self.name.lock().unwrap(),
                match &self.b {
                    Some(b) => b.test(),
                    None => "".to_string(),
                }
            )
        }
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    struct BImpl {
        pub name: Arc<std::sync::Mutex<String>>,
    }

    #[dill::component]
    #[dill::interface(dyn B)]
    #[dill::scope(dill::scopes::Transaction)]
    impl BImpl {
        pub fn new() -> Self {
            Self {
                name: Default::default(),
            }
        }

        pub fn set_name(&self, name: impl Into<String>) {
            *self.name.lock().unwrap() = name.into();
        }
    }

    impl B for BImpl {
        fn test(&self) -> String {
            format!("{}", self.name.lock().unwrap().as_str())
        }
    }

    let base = dill::Catalog::builder()
        .add::<AImpl>()
        .add::<BImpl>()
        .build();

    // Error if used outside of a transaction
    assert_matches!(
        base.get_one::<BImpl>().err(),
        Some(dill::InjectionError::Unregistered(_))
    );

    // TX: 1
    {
        let tx = base
            .builder_chained()
            .add_value(dill::scopes::TransactionCache::new())
            .build();

        let a = tx.get_one::<AImpl>().unwrap();
        let b = tx.get_one::<BImpl>().unwrap();
        assert_eq!(a.test(), "::");

        // `b` is same instance as injected into `a`
        a.set_name("foo");
        b.set_name("bar");
        assert_eq!(a.test(), "foo::bar");

        // Different A, but B is reused
        let a = tx.get_one::<AImpl>().unwrap();
        assert_eq!(a.test(), "::bar");
    }

    // TX: 2
    {
        let tx = base
            .builder_chained()
            .add_value(dill::scopes::TransactionCache::new())
            .build();

        // B instance was dropped with TX 1 and will be re-created
        let a = tx.get_one::<AImpl>().unwrap();
        assert_eq!(a.test(), "::");
    }
}
