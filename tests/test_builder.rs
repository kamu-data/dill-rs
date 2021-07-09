use std::{any::TypeId, sync::Arc};

use dill::*;

#[test]
fn test_type_info() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl;

    impl A for AImpl {
        fn test(&self) -> String {
            "foo".to_owned()
        }
    }

    let mut cat = Catalog::new();
    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();

    let builders: Vec<_> = cat.builders_for::<dyn A>().collect();
    assert_eq!(builders.len(), 1);

    let builder = builders.into_iter().next().unwrap();
    assert_eq!(builder.instance_type_id(), TypeId::of::<AImpl>());
    assert_eq!(
        builder.instance_type_name(),
        "test_builder::test_type_info::AImpl"
    );
}

#[test]
fn test_with_args_by_value() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    #[component]
    struct AImpl {
        host: String,
        port: i32,
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}::{}", self.host, self.port)
        }
    }

    let mut cat = Catalog::new();
    cat.add_builder(
        builder_for::<AImpl>()
            .with_host("foo".to_owned())
            .with_port(8080),
    );
    cat.bind::<dyn A, AImpl>().unwrap();
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo::8080");

    let mut cat = Catalog::new();
    cat.add_builder(
        builder_for::<AImpl>()
            .with_host_fn(|_| Ok("bar".to_owned()))
            .with_port_fn(|_| Ok(8080)),
    );
    cat.bind::<dyn A, AImpl>().unwrap();
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bar::8080");
}

#[test]
fn test_with_args_by_ref() {
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

    struct BImpl1;

    impl B for BImpl1 {
        fn test(&self) -> String {
            "bimpl1".to_owned()
        }
    }

    #[component]
    struct BImpl2;

    impl B for BImpl2 {
        fn test(&self) -> String {
            "bimpl2".to_owned()
        }
    }

    let mut cat = Catalog::new();
    cat.add_builder(builder_for::<AImpl>().with_b(Arc::new(BImpl1)));
    cat.bind::<dyn A, AImpl>().unwrap();
    cat.add::<BImpl2>();
    cat.bind::<dyn B, BImpl2>().unwrap();
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl1");

    let mut cat = Catalog::new();
    cat.add_builder(builder_for::<AImpl>().with_b_fn(|_| Ok(Arc::new(BImpl1))));
    cat.bind::<dyn A, AImpl>().unwrap();
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl1");
}

#[test]
fn test_new_ctor() {
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

    let mut cat = Catalog::new();
    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();
    cat.add::<BImpl>();
    cat.bind::<dyn B, BImpl>().unwrap();
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::bimpl::foo");
}

#[test]
fn test_new_ctor_cloned() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        b: B,
    }

    #[component]
    impl AImpl {
        pub fn new(bee: B) -> Self {
            Self { b: bee }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b.0)
        }
    }

    #[derive(Clone)]
    struct B(String);

    let mut cat = Catalog::new();
    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();
    cat.add_value(B("foo".to_owned()));
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo");
}

#[test]
fn test_new_ctor_by_ref() {
    trait A: Send + Sync {
        fn test(&self) -> String;
    }

    struct AImpl {
        b: String,
    }

    #[component]
    impl AImpl {
        pub fn new(bee: &B) -> Self {
            Self { b: bee.0.clone() }
        }
    }

    impl A for AImpl {
        fn test(&self) -> String {
            format!("aimpl::{}", self.b)
        }
    }

    struct B(String);

    let mut cat = Catalog::new();
    cat.add::<AImpl>();
    cat.bind::<dyn A, AImpl>().unwrap();
    cat.add_value(B("foo".to_owned()));
    let inst = cat.get::<OneOf<dyn A>>().unwrap();
    assert_eq!(inst.test(), "aimpl::foo");
}
