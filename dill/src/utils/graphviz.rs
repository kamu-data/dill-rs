use std::fmt::Write;

use crate::*;

fn get_type_name(i: &TypeInfo) -> String {
    let iang = i.type_name.find('<').unwrap_or(i.type_name.len());
    let icol = i.type_name[0..iang].rfind("::").map(|i| i + 2).unwrap_or(0);

    format!("\"{}\"", &i.type_name[icol..iang])
}

fn get_spec_name(i: &DependencyInfo) -> String {
    let spec = i
        .spec
        .type_name
        .replace(i.type_info.type_name, "")
        .replace("dill::specs::", "");

    match spec.as_str() {
        "OneOf<>" => String::new(),
        "AllOf<>" => "*".to_string(),
        "Maybe<OneOf<>>" => "?".to_string(),
        "Lazy<OneOf<>>" => "lazy".to_string(),
        _ => spec,
    }
}

pub fn render(cat: &Catalog) -> String {
    let mut s = String::new();

    writeln!(
        s,
        indoc::indoc!(
            r#"
            digraph Catalog {{
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
            "#
        )
    )
    .unwrap();

    let mut builders: Vec<_> = cat.builders().collect();
    builders.sort_by_key(|b| b.instance_type().type_name);

    for b in &builders {
        let inst = b.instance_type();

        let mut ifaces = b.interfaces_get_all();
        ifaces.sort_by_key(|i| i.type_name);

        let mut deps = b.dependencies_get_all();
        deps.sort_by_key(|i| i.type_info.type_name);

        for iface in &ifaces {
            writeln!(
                s,
                "    {} -> {} [style=dashed, arrowhead=onormal]",
                get_type_name(&inst),
                get_type_name(iface)
            )
            .unwrap();
        }

        for dep in &deps {
            writeln!(
                s,
                "    {} -> {} [label=\"{}\", arrowhead=vee]",
                get_type_name(&inst),
                get_type_name(&dep.type_info),
                get_spec_name(dep)
            )
            .unwrap();
        }
    }

    writeln!(s, "}}").unwrap();
    s
}
