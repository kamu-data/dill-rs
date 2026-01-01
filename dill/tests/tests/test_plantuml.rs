use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_plantuml_render() {
    trait A: Send + Sync {}

    #[dill::component]
    #[dill::interface(dyn A)]
    struct AImpl {
        #[allow(dead_code)]
        b: Arc<dyn B>,
    }
    impl A for AImpl {}

    trait B: Send + Sync {}

    #[dill::component]
    #[dill::interface(dyn B)]
    struct BImpl;

    impl B for BImpl {}

    let cat = dill::Catalog::builder()
        .add::<AImpl>()
        .add::<BImpl>()
        .build();

    pretty_assertions::assert_eq!(
        dill::utils::plantuml::render(&cat),
        indoc::indoc!(
            r#"
            @startuml

            hide circle
            hide empty members
            hide <<i>> stereotype

            <style>
            class {
                .i {
                    FontColor blue
                }
            }
            </style>

            interface "A" <<i>>
            interface "B" <<i>>
            class "AImpl"
            class "BImpl"
            "A" <|-- "AImpl"
            "AImpl"  --> "B"
            "B" <|-- "BImpl"
            @enduml
            "#
        ),
    );
}
