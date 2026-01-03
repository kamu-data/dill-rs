use std::sync::Arc;

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

#[test]
fn test_macro_explicit_args_field() {
    use dill::*;

    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl {
        b: Arc<dyn B>,

        #[component(explicit)]
        suffix: String,
    }
    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.b.test(), self.suffix)
        }
    }

    #[component]
    #[interface(dyn B)]
    struct BImpl;
    impl B for BImpl {
        fn test(&self) -> String {
            "bimpl".to_owned()
        }
    }

    let cat = Catalog::builder()
        .add::<BImpl>()
        .add_builder(AImpl::builder("foo".to_string()))
        .build();

    let a = cat.get_one::<AImpl>().unwrap();
    assert_eq!(a.test(), "aimpl::bimpl::foo");
}

#[test]
fn test_macro_explicit_args_ctor() {
    use dill::*;

    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        b: Arc<dyn B>,
        suffix: Arc<dyn Fn() -> String + Send + Sync>,
    }
    #[component]
    impl AImpl {
        pub fn new(
            b: Arc<dyn B>,
            #[component(explicit)] suffix: Arc<dyn Fn() -> String + Send + Sync>,
        ) -> Self {
            Self { b, suffix }
        }
    }
    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.b.test(), (self.suffix)())
        }
    }

    #[component]
    #[interface(dyn B)]
    struct BImpl;
    impl B for BImpl {
        fn test(&self) -> String {
            "bimpl".to_owned()
        }
    }

    let cat = Catalog::builder()
        .add::<BImpl>()
        .add_builder(AImpl::builder(Arc::new(|| "foo".to_string())))
        .build();

    let a = cat.get_one::<AImpl>().unwrap();
    assert_eq!(a.test(), "aimpl::bimpl::foo");
}

#[test]
fn test_macro_generates_new() {
    use dill::*;

    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    trait B: Send + Sync {
        fn test(&self) -> String;
    }

    // With generated new
    #[component]
    struct AImpl1 {
        b: Arc<dyn B>,
    }
    impl A for AImpl1 {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b.test())
        }
    }

    // With custom new
    #[component(no_new)]
    struct AImpl2 {
        #[expect(unused)]
        b: Arc<dyn B>,
    }
    impl AImpl2 {
        // This would cause compile error if `no_new` was not respected
        #[expect(unused)]
        pub fn new() -> Self {
            unreachable!()
        }
    }

    #[component]
    struct BImpl;
    impl B for BImpl {
        fn test(&self) -> String {
            "bimpl".to_owned()
        }
    }

    let a = AImpl1::new(Arc::new(BImpl::new()));
    assert_eq!(a.test(), "aimpl::bimpl");
}
