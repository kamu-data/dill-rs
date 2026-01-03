use std::sync::Arc;

use dill::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_add_builder_arc() {
    let mut cat = CatalogBuilder::new();
    cat.add_builder(Arc::new("foo".to_owned()));
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_add_builder_fn() {
    let mut cat = CatalogBuilder::new();
    cat.add_builder(|| Arc::new("foo".to_owned()));
    let cat = cat.build();

    let val = cat.get_one::<String>().unwrap();
    assert_eq!(val.as_ref(), "foo");
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[should_panic]
fn test_add_impl_twice_panics() {
    let mut cat = CatalogBuilder::new();
    cat.add_value("foo".to_owned());
    cat.add_value("foo".to_owned());
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[should_panic]
fn test_bind_with_no_impl_panics() {
    trait A {}

    #[component]
    struct AImpl;

    impl A for AImpl {}

    CatalogBuilder::new().bind::<dyn A, AImpl>();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_self_injection_weak_ref() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl1 {
        catalog: CatalogWeakRef,
    }

    impl A for AImpl1 {
        fn test(&self) -> String {
            let b = self.catalog.get_one::<dyn B>().unwrap();
            format!("aimpl::{}", b.test())
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
        fn new(catalog: CatalogWeakRef) -> Self {
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_self_injection_by_val() {
    #[component]
    struct A {
        #[allow(unused)]
        catalog: Catalog,
    }

    let cat = CatalogBuilder::new().add::<A>().build();
    cat.get_one::<A>().unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_self_injection_singleton_cleanup() {
    static INSTANCE_COUNT: std::sync::Mutex<i32> = std::sync::Mutex::new(0);

    struct A {
        #[allow(dead_code)]
        catalog: dill::CatalogWeakRef,
    }

    #[dill::component]
    #[dill::scope(dill::scopes::Singleton)]
    impl A {
        pub fn new(catalog: CatalogWeakRef) -> Self {
            (*INSTANCE_COUNT.lock().unwrap()) += 1;
            Self { catalog }
        }
    }

    impl Drop for A {
        fn drop(&mut self) {
            (*INSTANCE_COUNT.lock().unwrap()) -= 1;
        }
    }

    {
        let cat = CatalogBuilder::new().add::<A>().build();

        {
            let _inst = cat.get_one::<A>().unwrap();

            {
                // We have the instance
                let instance_count = *INSTANCE_COUNT.lock().unwrap();
                assert_eq!(instance_count, 1);
            }
        }

        {
            // Singleton still caches the instance
            let instance_count = *INSTANCE_COUNT.lock().unwrap();
            assert_eq!(instance_count, 1);
        }
    }

    {
        // Catalog drops and releases the instance cached in a Singleton scope
        let instance_count = *INSTANCE_COUNT.lock().unwrap();
        assert_eq!(instance_count, 0);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_self_injection_into_singleton_via_chained_catalog() {
    static INSTANCE_COUNT: std::sync::Mutex<i32> = std::sync::Mutex::new(0);

    #[dill::component]
    struct A {
        b: Arc<B>,
    }

    impl A {
        fn run(&self) -> String {
            self.b.run()
        }
    }

    struct B {
        catalog: dill::CatalogWeakRef,
    }

    #[dill::component]
    #[dill::scope(dill::scopes::Singleton)]
    impl B {
        pub fn new(catalog: CatalogWeakRef) -> Self {
            (*INSTANCE_COUNT.lock().unwrap()) += 1;
            Self { catalog }
        }

        fn run(&self) -> String {
            let c = self.catalog.get_one::<C>().unwrap();
            c.run()
        }
    }

    impl Drop for B {
        fn drop(&mut self) {
            (*INSTANCE_COUNT.lock().unwrap()) -= 1;
        }
    }

    #[dill::component]
    struct C {}

    impl C {
        fn run(&self) -> String {
            "hello".to_string()
        }
    }

    {
        let base_cat = CatalogBuilder::new().add::<B>().add::<C>().build();

        {
            let chained_cat = base_cat.builder_chained().add::<A>().build();

            let a = chained_cat.get_one::<A>().unwrap();
            assert_eq!(a.run(), "hello");

            {
                // We have the instance
                let instance_count = *INSTANCE_COUNT.lock().unwrap();
                assert_eq!(instance_count, 1);
            }
        }

        {
            // Singleton still caches the instance
            let instance_count = *INSTANCE_COUNT.lock().unwrap();
            assert_eq!(instance_count, 1);
        }

        {
            let chained_cat = base_cat.builder_chained().add::<A>().build();

            let a = chained_cat.get_one::<A>().unwrap();
            assert_eq!(a.run(), "hello");

            {
                // We reuse the instance
                let instance_count = *INSTANCE_COUNT.lock().unwrap();
                assert_eq!(instance_count, 1);
            }
        }
    }

    {
        // Catalog drops and releases the instance cached in a Singleton scope
        let instance_count = *INSTANCE_COUNT.lock().unwrap();
        assert_eq!(instance_count, 0);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_catalog_binds_interfaces_for_builder_with_impl_without_explicit_args() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    #[interface(dyn A)]
    struct AImpl;

    impl A for AImpl {
        fn test(&self) -> String {
            "aimpl".to_string()
        }
    }

    // NOTE: In this case it is more correct to use `.add::<AImpl>()`,
    //       but for the sake of the test we use `add_builder()` method
    let catalog = CatalogBuilder::new().add_builder(AImpl::builder()).build();

    let a_impl = catalog.get_one::<AImpl>().unwrap();
    assert_eq!(a_impl.test(), "aimpl");

    let a = catalog.get_one::<dyn A>().unwrap();
    assert_eq!(a.test(), "aimpl");
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_catalog_skips_interfaces_binding_for_builder_with_impl_without_explicit_args() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    #[interface(dyn A)]
    struct AImpl;

    impl A for AImpl {
        fn test(&self) -> String {
            "aimpl".to_string()
        }
    }

    let catalog = CatalogBuilder::new()
        .add_builder(AImpl::builder().without_default_interfaces())
        .build();

    let a_impl = catalog.get_one::<AImpl>().unwrap();
    assert_eq!(a_impl.test(), "aimpl");

    match catalog.get_one::<dyn A>() {
        Err(InjectionError::Unregistered(_)) => {}
        _ => panic!("Expected an unregistered error"),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_catalog_binds_interfaces_for_builder_with_explicit_args() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    #[interface(dyn A)]
    struct AImpl {
        #[component(explicit)]
        suffix: String,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.suffix)
        }
    }

    let catalog = CatalogBuilder::new()
        .add_builder(AImpl::builder("foo".to_string()))
        .build();

    let a_impl = catalog.get_one::<AImpl>().unwrap();
    assert_eq!(a_impl.test(), "aimpl::foo");

    let a = catalog.get_one::<dyn A>().unwrap();
    assert_eq!(a.test(), "aimpl::foo");
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_catalog_skips_interfaces_binding_for_builder_with_explicit_args() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    #[interface(dyn A)]
    struct AImpl {
        #[component(explicit)]
        suffix: String,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.suffix)
        }
    }

    let catalog = CatalogBuilder::new()
        .add_builder(AImpl::builder("foo".to_string()).without_default_interfaces())
        .build();

    let a_impl = catalog.get_one::<AImpl>().unwrap();
    assert_eq!(a_impl.test(), "aimpl::foo");

    match catalog.get_one::<dyn A>() {
        Err(InjectionError::Unregistered(_)) => {}
        _ => panic!("Expected an unregistered error"),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_catalog_binds_interfaces_for_builder_does_not_require_an_explicit_bind() {
    trait A: Send + Sync {}

    #[component]
    #[interface(dyn A)]
    struct AImpl {}

    impl A for AImpl {}

    let catalog = CatalogBuilder::new()
        .add_builder(AImpl::builder())
        .bind::<dyn A, AImpl>()
        .build();

    let _a_impl = catalog.get_one::<AImpl>().unwrap();

    match catalog.get_one::<dyn A>() {
        Err(InjectionError::Ambiguous(_)) => {}
        _ => panic!("Expected an ambiguous error"),
    }
}
