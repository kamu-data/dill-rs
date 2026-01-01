use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_graphviz_render() {
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
        dill::utils::graphviz::render(&cat),
        indoc::indoc!(
            r#"
            digraph Catalog {
                rankdir=LR;
                fontsize=8;
                fontname="Roboto";

                node [
                    shape=box,
                    style=filled,
                    fillcolor=white,
                    fontname="Roboto",
                    fontsize=8
                ];

                edge [
                    fontname="Roboto",
                    fontsize=8
                ];

                "AImpl" -> "A" [style=dashed, arrowhead=onormal]
                "AImpl" -> "B" [label="", arrowhead=vee]
                "BImpl" -> "B" [style=dashed, arrowhead=onormal]
            }
            "#
        ),
    );
}
