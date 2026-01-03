use std::assert_matches::assert_matches;
use std::sync::Arc;

use dill::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
        Err(ValidationError { errors }) if matches!(
            &errors[..],
            [InjectionError::Unregistered(u)]
            if u.dep_type.name == "dyn unit::tests::test_validation::test_validate_static_graph::B"
        )
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_validate_optional() {
    #[allow(dead_code)]
    #[component]
    struct A {
        foo: Option<i32>,
    }

    let mut b = Catalog::builder();
    b.add::<A>();
    b.validate().unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_validate_bound_fields() {
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_validate_catalog_inject() {
    #[allow(dead_code)]
    struct A {
        catalog: CatalogWeakRef,
    }

    #[component]
    impl A {
        pub fn new(catalog: CatalogWeakRef, _catalog_ref: &Catalog) -> Self {
            Self { catalog }
        }
    }

    let mut b = CatalogBuilder::new();
    b.add::<A>();

    b.validate().unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_validate_scope_inversion_transient_in_singleton() {
    #[dill::component]
    #[dill::scope(dill::scopes::Singleton)]
    struct A {
        #[allow(dead_code)]
        b: Arc<B>,
    }

    #[dill::component]
    struct B;

    let mut b = CatalogBuilder::new();
    b.add::<A>();
    b.add::<B>();

    let res = b.validate();
    assert_matches!(
        &res,
        Err(ValidationError { errors }) if matches!(
            &errors[..],
            [InjectionError::ScopeInversion(..)],
        )
    );

    pretty_assertions::assert_eq!(
        res.err().unwrap().errors[0].to_string(),
        indoc::indoc!(
            r#"
            Scope inversion: unit::tests::test_validation::test_validate_scope_inversion_transient_in_singleton::A in dill::scopes::Singleton scope injects unit::tests::test_validation::test_validate_scope_inversion_transient_in_singleton::B in dill::scopes::Transient scope
            Injection stack:
              0: Build:   unit::tests::test_validation::test_validate_scope_inversion_transient_in_singleton::A <dill::scopes::Singleton>
              1: Resolve: dill::specs::OneOf<unit::tests::test_validation::test_validate_scope_inversion_transient_in_singleton::B>
            "#
        )
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_validate_scope_inversion_agnostic_in_singleton() {
    #[dill::component]
    #[dill::scope(dill::scopes::Singleton)]
    struct A {
        #[allow(dead_code)]
        b: Arc<B>,
    }

    #[dill::component]
    #[dill::scope(dill::scopes::Agnostic)]
    struct B;

    let mut b = CatalogBuilder::new();
    b.add::<A>();
    b.add::<B>();

    b.validate().unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_validate_scope_inversion_tx_in_singleton() {
    #[dill::component]
    #[dill::scope(dill::scopes::Singleton)]
    struct A {
        #[allow(dead_code)]
        b: Arc<B>,
    }

    #[dill::component]
    #[dill::scope(dill::scopes::Transaction)]
    struct B;

    let mut b = CatalogBuilder::new();
    b.add::<A>();
    b.add::<B>();

    let res = b.validate();
    assert_matches!(
        &res,
        Err(ValidationError { errors }) if matches!(
            &errors[..],
            [InjectionError::ScopeInversion(_)],
        )
    );

    pretty_assertions::assert_eq!(
        res.err().unwrap().errors[0].to_string(),
        indoc::indoc!(
            r#"
            Scope inversion: unit::tests::test_validation::test_validate_scope_inversion_tx_in_singleton::A in dill::scopes::Singleton scope injects unit::tests::test_validation::test_validate_scope_inversion_tx_in_singleton::B in dill::scopes::Cached<dill::scopes::TransactionCache> scope
            Injection stack:
              0: Build:   unit::tests::test_validation::test_validate_scope_inversion_tx_in_singleton::A <dill::scopes::Singleton>
              1: Resolve: dill::specs::OneOf<unit::tests::test_validation::test_validate_scope_inversion_tx_in_singleton::B>
            "#
        )
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
