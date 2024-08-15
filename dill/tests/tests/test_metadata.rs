use dill::{Builder, CatalogBuilder};

#[test]
fn test_metadata() {
    #[allow(dead_code)]
    #[derive(Debug)]
    struct EventHandlerDesc {
        event_type: &'static str,
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    struct Unrelated {
        event_type: &'static str,
    }

    trait EventHandler: Send + Sync {
        fn on_event(&self, event: &str) -> String;
    }

    #[dill::component]
    #[dill::interface(dyn EventHandler)]
    #[dill::meta(EventHandlerDesc { event_type: "A"})]
    struct EventHandlerA;
    impl EventHandler for EventHandlerA {
        fn on_event(&self, event: &str) -> String {
            format!("HandlerA: {event}")
        }
    }

    #[dill::component]
    #[dill::interface(dyn EventHandler)]
    #[dill::meta(EventHandlerDesc { event_type: "B"})]
    struct EventHandlerB;
    impl EventHandler for EventHandlerB {
        fn on_event(&self, event: &str) -> String {
            format!("HandlerB: {event}")
        }
    }

    #[dill::component]
    #[dill::interface(dyn EventHandler)]
    #[dill::meta(EventHandlerDesc { event_type: "A"})]
    #[dill::meta(EventHandlerDesc { event_type: "B"})]
    #[dill::meta(Unrelated { event_type: "X"})]
    struct EventHandlerAB;
    impl EventHandler for EventHandlerAB {
        fn on_event(&self, event: &str) -> String {
            format!("HandlerAB: {event}")
        }
    }

    let cat = dill::CatalogBuilder::new()
        .add::<EventHandlerA>()
        .add::<EventHandlerB>()
        .add::<EventHandlerAB>()
        .build();

    // Check low-level interface

    let mut metas = Vec::new();
    for b in cat.builders_for::<dyn EventHandler>() {
        b.metadata(&mut |meta| {
            if let Some(meta) = meta.downcast_ref::<EventHandlerDesc>() {
                metas.push((b.instance_type_name(), meta.event_type));
            }
            true
        });
    }

    metas.sort();
    assert_eq!(
        metas,
        [
            (
                "unit::tests::test_metadata::test_metadata::EventHandlerA",
                "A"
            ),
            (
                "unit::tests::test_metadata::test_metadata::EventHandlerAB",
                "A"
            ),
            (
                "unit::tests::test_metadata::test_metadata::EventHandlerAB",
                "B"
            ),
            (
                "unit::tests::test_metadata::test_metadata::EventHandlerB",
                "B"
            )
        ]
    );

    // Check helper methods
    let mut res = cat
        .builders_for_with_meta::<dyn EventHandler, _>(|desc: &EventHandlerDesc| {
            desc.event_type == "B"
        })
        .map(|b| b.instance_type_name())
        .collect::<Vec<_>>();

    res.sort();
    assert_eq!(
        res,
        [
            "unit::tests::test_metadata::test_metadata::EventHandlerAB",
            "unit::tests::test_metadata::test_metadata::EventHandlerB"
        ]
    );

    // Check filtering can be applied on `DependencySpec` level

    struct EventHandlersForA;

    impl dill::DependencySpec for EventHandlersForA {
        type ReturnType = Vec<std::sync::Arc<dyn EventHandler>>;

        fn get(cat: &dill::Catalog) -> Result<Self::ReturnType, dill::InjectionError> {
            cat.builders_for_with_meta::<dyn EventHandler, _>(|desc: &EventHandlerDesc| {
                desc.event_type == "A"
            })
            .map(|b| b.get(cat))
            .collect()
        }

        fn check(_cat: &dill::Catalog) -> Result<(), dill::InjectionError> {
            unimplemented!()
        }
    }

    let mut res = cat
        .get::<EventHandlersForA>()
        .unwrap()
        .into_iter()
        .map(|h| h.on_event("test"))
        .collect::<Vec<_>>();

    res.sort();
    assert_eq!(res, ["HandlerA: test", "HandlerAB: test"]);

    let chained_cat = CatalogBuilder::new_chained(&cat).build();
    let mut res = chained_cat
        .builders_for_with_meta::<dyn EventHandler, _>(|desc: &EventHandlerDesc| {
            desc.event_type == "B"
        })
        .map(|b| b.instance_type_name())
        .collect::<Vec<_>>();

    res.sort();
    assert_eq!(
        res,
        [
            "unit::tests::test_metadata::test_metadata::EventHandlerAB",
            "unit::tests::test_metadata::test_metadata::EventHandlerB"
        ]
    );
}
