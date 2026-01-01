use quote::ToTokens;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) enum InjectionType {
    Catalog,
    CatalogRef,
    Arc { inner: syn::Type },
    Reference { inner: syn::Type },
    Option { element: Box<InjectionType> },
    Vec { item: Box<InjectionType> },
    Lazy { element: Box<InjectionType> },
    Value { typ: syn::Type },
}

pub(crate) fn deduce_injection_type(typ: &syn::Type) -> InjectionType {
    if is_catalog(typ) {
        InjectionType::Catalog
    } else if let Some(inner) = strip_reference(typ) {
        if is_catalog(&inner) {
            InjectionType::CatalogRef
        } else {
            InjectionType::Reference { inner }
        }
    } else if let Some(inner) = get_arc_element_type(typ) {
        InjectionType::Arc { inner }
    } else if let Some(elem_typ) = get_option_element_type(typ) {
        InjectionType::Option {
            element: Box::new(deduce_injection_type(&elem_typ)),
        }
    } else if let Some(elem_typ) = get_vec_item_type(typ) {
        InjectionType::Vec {
            item: Box::new(deduce_injection_type(&elem_typ)),
        }
    } else if let Some(elem_typ) = get_lazy_element_type(typ) {
        InjectionType::Lazy {
            element: Box::new(deduce_injection_type(&elem_typ)),
        }
    } else {
        InjectionType::Value { typ: typ.clone() }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_catalog(typ: &syn::Type) -> bool {
    let syn::Type::Path(typepath) = typ else {
        return false;
    };

    typepath.qself.is_none() && typepath.path.segments.last().unwrap().ident == "Catalog"
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn strip_reference(typ: &syn::Type) -> Option<syn::Type> {
    match typ {
        syn::Type::Reference(r) => Some(r.elem.as_ref().clone()),
        _ => None,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_arc_element_type(typ: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(typepath) = typ else {
        panic!("Expected a Type::Path");
    };

    if typepath.qself.is_some() || typepath.path.segments.last().unwrap().ident != "Arc" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) =
        &typepath.path.segments.last().unwrap().arguments
    else {
        return None;
    };

    Some(syn::parse2(args.args.to_token_stream()).unwrap())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_option_element_type(typ: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(typepath) = typ else {
        panic!("Expected a Type::Path");
    };

    if typepath.qself.is_some() || &typepath.path.segments.last().unwrap().ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) =
        &typepath.path.segments.last().unwrap().arguments
    else {
        return None;
    };

    Some(syn::parse2(args.args.to_token_stream()).unwrap())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_lazy_element_type(typ: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(typepath) = typ else {
        panic!("Expected a Type::Path");
    };

    if typepath.qself.is_some() || &typepath.path.segments.last().unwrap().ident != "Lazy" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) =
        &typepath.path.segments.last().unwrap().arguments
    else {
        return None;
    };

    Some(syn::parse2(args.args.to_token_stream()).unwrap())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_vec_item_type(typ: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(typepath) = typ else {
        panic!("Expected a Type::Path");
    };

    if typepath.qself.is_some() || typepath.path.segments.last().unwrap().ident != "Vec" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) =
        &typepath.path.segments.last().unwrap().arguments
    else {
        return None;
    };

    Some(syn::parse2(args.args.to_token_stream()).unwrap())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
