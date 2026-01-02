use std::fmt::Write;

use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: Currently grouping by packages produces poor visual results
fn get_type_package(_i: &TypeInfo) -> Option<String> {
    // if let Some((c, _)) = i.type_name.replace("dyn ", "").split_once("::") {
    //     Some(format!("\"{c}\""))
    // } else {
    //     None
    // }
    None
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_type_name(i: &TypeInfo) -> String {
    let iang = i.name.find('<').unwrap_or(i.name.len());
    let icol = i.name[0..iang].rfind("::").map(|i| i + 2).unwrap_or(0);

    format!("\"{}\"", &i.name[icol..iang])
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_type_scope(i: &TypeInfo) -> String {
    let name = if i.id == std::any::TypeId::of::<Transient>() {
        "".to_string()
    } else {
        i.name.to_lowercase().replace("dill::scopes::", "")
    };

    if name.is_empty() {
        String::new()
    } else {
        format!(" <<{name}>>")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_spec_name(i: &DependencyInfo) -> String {
    let spec = i
        .spec
        .name
        .replace(i.iface.name, "")
        .replace("dill::specs::", "");

    let s = match spec.as_str() {
        "OneOf<>" => String::new(),
        "AllOf<>" => "*".to_string(),
        "Maybe<OneOf<>>" => "?".to_string(),
        "Lazy<OneOf<>>" => "lazy".to_string(),
        _ => spec,
    };

    if s.is_empty() { s } else { format!("\"{s}\"") }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn render(cat: &Catalog) -> String {
    let mut s = String::new();

    writeln!(
        s,
        indoc::indoc!(
            r#"
            @startuml

            hide circle
            hide empty members
            hide <<i>> stereotype

            <style>
            class {{
                .i {{
                    FontColor blue
                }}
            }}
            </style>
            "#
        )
    )
    .unwrap();

    let ifaces: std::collections::HashSet<(Option<String>, String)> = cat
        .builders()
        .flat_map(|b| b.interfaces_get_all())
        .map(|i| (get_type_package(&i), get_type_name(&i)))
        .collect();
    let mut ifaces: Vec<_> = ifaces.into_iter().collect();
    ifaces.sort();

    for group in ifaces.chunk_by(|a, b| a.0 == b.0) {
        if let Some(p) = &group[0].0 {
            writeln!(s, "package {p} {{").unwrap();
        }

        for (_, name) in group {
            writeln!(s, "interface {name} <<i>>").unwrap();
        }

        if group[0].0.is_some() {
            writeln!(s, "}}").unwrap();
        }
    }

    let mut instances: Vec<(Option<String>, String, String)> = cat
        .builders()
        .map(|b| {
            (
                get_type_package(&b.instance_type()),
                get_type_name(&b.instance_type()),
                get_type_scope(&b.scope_type()),
            )
        })
        .collect();
    instances.sort();

    for group in instances.chunk_by(|a, b| a.0 == b.0) {
        if let Some(p) = &group[0].0 {
            writeln!(s, "package {p} {{").unwrap();
        }

        for (_, name, scope) in group {
            writeln!(s, "class {name}{scope}").unwrap();
        }

        if group[0].0.is_some() {
            writeln!(s, "}}").unwrap();
        }
    }

    let mut builders: Vec<_> = cat.builders().collect();
    builders.sort_by_key(|b| b.instance_type().name);

    for b in &builders {
        let inst = b.instance_type();

        let mut ifaces = b.interfaces_get_all();
        ifaces.sort_by_key(|i| i.name);

        let mut deps = b.dependencies_get_all();
        deps.sort_by_key(|i| i.iface.name);

        for iface in &ifaces {
            writeln!(s, "{} <|-- {}", get_type_name(iface), get_type_name(&inst)).unwrap();
        }

        for dep in &deps {
            writeln!(
                s,
                "{} {} --> {}",
                get_type_name(&inst),
                get_spec_name(dep),
                get_type_name(&dep.iface)
            )
            .unwrap();
        }
    }

    writeln!(s, "@enduml").unwrap();
    s
}
