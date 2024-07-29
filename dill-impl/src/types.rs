use quote::ToTokens;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) enum InjectionType {
    Arc { inner: syn::Type },
    Reference { inner: syn::Type },
    Option { element: Box<InjectionType> },
    Vec { item: Box<InjectionType> },
    Value { typ: syn::Type },
}

pub(crate) fn deduce_injection_type(typ: &syn::Type) -> InjectionType {
    if is_reference(typ) {
        InjectionType::Reference {
            inner: strip_reference(typ),
        }
    } else if is_smart_ptr(typ) {
        InjectionType::Arc {
            inner: strip_smart_ptr(typ),
        }
    } else if is_option(typ) {
        InjectionType::Option {
            element: Box::new(deduce_injection_type(&get_option_element_type(typ))),
        }
    } else if is_vec(typ) {
        InjectionType::Vec {
            item: Box::new(deduce_injection_type(&get_vec_item_type(typ))),
        }
    } else {
        InjectionType::Value { typ: typ.clone() }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_reference(typ: &syn::Type) -> bool {
    matches!(typ, syn::Type::Reference(_))
}

pub(crate) fn strip_reference(typ: &syn::Type) -> syn::Type {
    match typ {
        syn::Type::Reference(r) => r.elem.as_ref().clone(),
        _ => typ.clone(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_smart_ptr(typ: &syn::Type) -> bool {
    let syn::Type::Path(typepath) = typ else {
        return false;
    };

    if typepath.qself.is_some() || typepath.path.segments.len() != 1 {
        return false;
    }

    &typepath.path.segments[0].ident == "Arc"
}

pub(crate) fn strip_smart_ptr(typ: &syn::Type) -> syn::Type {
    match typ {
        syn::Type::Path(typepath) if typepath.qself.is_none() => {
            match typepath.path.segments.first() {
                Some(seg) if &seg.ident == "Arc" => match seg.arguments {
                    syn::PathArguments::AngleBracketed(ref args) => {
                        syn::parse2(args.args.to_token_stream()).unwrap()
                    }
                    _ => typ.clone(),
                },
                _ => typ.clone(),
            }
        }
        _ => typ.clone(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_option(typ: &syn::Type) -> bool {
    let syn::Type::Path(typepath) = typ else {
        return false;
    };

    if typepath.qself.is_some() || typepath.path.segments.len() != 1 {
        return false;
    }

    &typepath.path.segments[0].ident == "Option"
}

pub(crate) fn get_option_element_type(typ: &syn::Type) -> syn::Type {
    let syn::Type::Path(typepath) = typ else {
        panic!("Type is not an Option")
    };

    assert!(typepath.qself.is_none());
    assert_eq!(typepath.path.segments.len(), 1);
    assert_eq!(&typepath.path.segments[0].ident, "Option");

    let syn::PathArguments::AngleBracketed(args) = &typepath.path.segments[0].arguments else {
        panic!("No generic type specifier found in Option")
    };
    syn::parse2(args.args.to_token_stream()).unwrap()
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_vec(typ: &syn::Type) -> bool {
    let syn::Type::Path(typepath) = typ else {
        return false;
    };

    if typepath.qself.is_some() || typepath.path.segments.len() != 1 {
        return false;
    }

    &typepath.path.segments[0].ident == "Vec"
}

pub(crate) fn get_vec_item_type(typ: &syn::Type) -> syn::Type {
    let syn::Type::Path(typepath) = typ else {
        panic!("Type is not a Vec")
    };

    assert!(typepath.qself.is_none());
    assert_eq!(typepath.path.segments.len(), 1);
    assert_eq!(&typepath.path.segments[0].ident, "Vec");

    let syn::PathArguments::AngleBracketed(args) = &typepath.path.segments[0].arguments else {
        panic!("No generic type specifier found in Vec")
    };
    syn::parse2(args.args.to_token_stream()).unwrap()
}

/////////////////////////////////////////////////////////////////////////////////////////
