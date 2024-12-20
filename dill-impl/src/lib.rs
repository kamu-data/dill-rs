extern crate proc_macro;

mod types;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use types::InjectionType;

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::Item = syn::parse(item).unwrap();
    let vis: syn::Visibility = syn::parse(attr).unwrap();
    match ast {
        syn::Item::Struct(struct_ast) => component_from_struct(struct_ast),
        syn::Item::Impl(impl_ast) => component_from_impl(vis, impl_ast),
        _ => {
            panic!("The #[component] macro can only be used on struct definition or an impl block")
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn scope(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn interface(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn meta(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/////////////////////////////////////////////////////////////////////////////////////////

fn component_from_struct(ast: syn::ItemStruct) -> TokenStream {
    let impl_name = &ast.ident;
    let impl_type = syn::parse2(quote! { #impl_name }).unwrap();
    let impl_generics = syn::parse2(quote! {}).unwrap();

    let args: Vec<_> = ast
        .fields
        .iter()
        .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
        .collect();

    let scope_type =
        get_scope(&ast.attrs).unwrap_or_else(|| syn::parse_str("::dill::Transient").unwrap());

    let interfaces = get_interfaces(&ast.attrs);
    let meta = get_meta(&ast.attrs);

    let mut gen: TokenStream = quote! { #ast }.into();
    let builder: TokenStream = implement_builder(
        &ast.vis,
        &impl_type,
        &impl_generics,
        scope_type,
        interfaces,
        meta,
        args,
        false,
    );

    gen.extend(builder);
    gen
}

/////////////////////////////////////////////////////////////////////////////////////////

fn component_from_impl(vis: syn::Visibility, ast: syn::ItemImpl) -> TokenStream {
    let impl_generics = &ast.generics;
    let impl_type = &ast.self_ty;
    let new = get_new(&ast.items).expect(
        "When using #[component] macro on the impl block it's expected to contain a new() \
         function. Otherwise use #[derive(Builder)] on the struct.",
    );

    let args: Vec<_> = new
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(targ) => targ,
            _ => panic!("Unexpected argument in new() function"),
        })
        .map(|arg| {
            (
                match arg.pat.as_ref() {
                    syn::Pat::Ident(ident) => ident.ident.clone(),
                    _ => panic!("Unexpected format of arguments in new() function"),
                },
                arg.ty.as_ref().clone(),
            )
        })
        .collect();

    let scope_type =
        get_scope(&ast.attrs).unwrap_or_else(|| syn::parse_str("::dill::Transient").unwrap());

    let interfaces = get_interfaces(&ast.attrs);
    let meta = get_meta(&ast.attrs);

    let mut gen: TokenStream = quote! { #ast }.into();
    let builder: TokenStream = implement_builder(
        &vis,
        impl_type,
        impl_generics,
        scope_type,
        interfaces,
        meta,
        args,
        true,
    );

    gen.extend(builder);
    gen
}

/////////////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::too_many_arguments)]
fn implement_builder(
    impl_vis: &syn::Visibility,
    impl_type: &syn::Type,
    _impl_generics: &syn::Generics,
    scope_type: syn::Path,
    interfaces: Vec<syn::Type>,
    meta: Vec<syn::ExprStruct>,
    args: Vec<(syn::Ident, syn::Type)>,
    has_new: bool,
) -> TokenStream {
    let builder_name = format_ident!("{}Builder", quote! { #impl_type }.to_string());

    let arg_name: Vec<_> = args.iter().map(|(name, _)| name).collect();

    let meta_provide: Vec<_> = meta
        .iter()
        .enumerate()
        .map(|(i, e)| implement_meta_provide(i, e))
        .collect();
    let meta_vars: Vec<_> = meta
        .iter()
        .enumerate()
        .map(|(i, e)| implement_meta_var(i, e))
        .collect();

    let mut arg_override_fn_field = Vec::new();
    let mut arg_override_fn_field_ctor = Vec::new();
    let mut arg_override_setters = Vec::new();
    let mut arg_prepare_dependency = Vec::new();
    let mut arg_provide_dependency = Vec::new();
    let mut arg_check_dependency = Vec::new();

    for (name, typ) in &args {
        let (
            override_fn_field,
            override_fn_field_ctor,
            override_setters,
            prepare_dependency,
            provide_dependency,
            check_dependency,
        ) = implement_arg(name, typ, &builder_name);

        arg_override_fn_field.push(override_fn_field);
        arg_override_fn_field_ctor.push(override_fn_field_ctor);
        arg_override_setters.push(override_setters);
        arg_prepare_dependency.push(prepare_dependency);
        arg_provide_dependency.push(provide_dependency);
        arg_check_dependency.push(check_dependency);
    }

    let ctor = if !has_new {
        quote! {
            #impl_type {
                #( #arg_name: #arg_provide_dependency, )*
            }
        }
    } else {
        quote! {
            #impl_type::new(#( #arg_provide_dependency, )*)
        }
    };

    let gen = quote! {
        impl ::dill::Component for #impl_type {
            type Builder = #builder_name;

            fn register(cat: &mut ::dill::CatalogBuilder) {
                cat.add_builder(Self::builder());

                #(
                    cat.bind::<#interfaces, #impl_type>();
                )*
            }

            fn builder() -> Self::Builder {
                #builder_name::new()
            }
        }

        #impl_vis struct #builder_name {
            scope: #scope_type,
            #(
                #arg_override_fn_field
            )*
        }

        impl #builder_name {
            #( #meta_vars )*

            pub fn new() -> Self {
                Self {
                    scope: #scope_type::new(),
                    #(
                        #arg_override_fn_field_ctor
                    )*
                }
            }

            #( #arg_override_setters )*

            fn build(&self, cat: &::dill::Catalog) -> Result<#impl_type, ::dill::InjectionError> {
                use ::dill::DependencySpec;
                #( #arg_prepare_dependency )*
                Ok(#ctor)
            }
        }

        impl ::dill::Builder for #builder_name {
            fn instance_type_id(&self) -> ::std::any::TypeId {
                ::std::any::TypeId::of::<#impl_type>()
            }

            fn instance_type_name(&self) -> &'static str {
                ::std::any::type_name::<#impl_type>()
            }

            fn interfaces(&self, clb: &mut dyn FnMut(&::dill::InterfaceDesc) -> bool) {
                #(
                    if !clb(&::dill::InterfaceDesc {
                        type_id: ::std::any::TypeId::of::<#interfaces>(),
                        type_name: ::std::any::type_name::<#interfaces>(),
                    }) { return }
                )*
            }

            fn metadata<'a>(&'a self, clb: & mut dyn FnMut(&'a dyn std::any::Any) -> bool) {
                #( #meta_provide )*
            }

            fn get(&self, cat: &::dill::Catalog) -> Result<::std::sync::Arc<dyn ::std::any::Any + Send + Sync>, ::dill::InjectionError> {
                Ok(::dill::TypedBuilder::get(self, cat)?)
            }

            fn check(&self, cat: &::dill::Catalog) -> Result<(), ::dill::ValidationError> {
                use ::dill::DependencySpec;

                let mut errors = Vec::new();
                #(
                if let Err(err) = #arg_check_dependency {
                    errors.push(err);
                }
                )*
                if errors.len() != 0 {
                    Err(::dill::ValidationError { errors })
                } else {
                    Ok(())
                }
            }
        }

        impl ::dill::TypedBuilder<#impl_type> for #builder_name {
            fn get(&self, cat: &::dill::Catalog) -> Result<std::sync::Arc<#impl_type>, ::dill::InjectionError> {
                use ::dill::Scope;

                if let Some(inst) = self.scope.get() {
                    return Ok(inst.downcast().unwrap());
                }

                let inst = ::std::sync::Arc::new(self.build(cat)?);

                self.scope.set(inst.clone());
                Ok(inst)
            }
        }
    };

    gen.into()
}

/////////////////////////////////////////////////////////////////////////////////////////

fn implement_arg(
    name: &syn::Ident,
    typ: &syn::Type,
    builder: &syn::Ident,
) -> (
    proc_macro2::TokenStream, // override_fn_field
    proc_macro2::TokenStream, // override_fn_field_ctor
    proc_macro2::TokenStream, // override_setters
    proc_macro2::TokenStream, // prepare_dependency
    proc_macro2::TokenStream, // provide_dependency
    proc_macro2::TokenStream, // check_dependency
) {
    let override_fn_name = format_ident!("arg_{}_fn", name);

    let injection_type = types::deduce_injection_type(typ);

    let override_fn_field = match &injection_type {
        InjectionType::Reference { .. } => proc_macro2::TokenStream::new(),
        _ => quote! {
            #override_fn_name: Option<Box<dyn Fn(&::dill::Catalog) -> Result<#typ, ::dill::InjectionError> + Send + Sync>>,
        },
    };

    let override_fn_field_ctor = match &injection_type {
        InjectionType::Reference { .. } => proc_macro2::TokenStream::new(),
        _ => quote! { #override_fn_name: None, },
    };

    let override_setters = match &injection_type {
        InjectionType::Reference { .. } => proc_macro2::TokenStream::new(),
        _ => {
            let setter_val_name = format_ident!("with_{}", name);
            let setter_fn_name = format_ident!("with_{}_fn", name);
            quote! {
                pub fn #setter_val_name(mut self, val: #typ) -> #builder {
                    self.#override_fn_name = Some(Box::new(move |_| Ok(val.clone())));
                    self
                }

                pub fn #setter_fn_name(
                    mut self,
                    fun: impl Fn(&::dill::Catalog) -> Result<#typ, ::dill::InjectionError> + 'static + Send + Sync
                ) -> #builder {
                    self.#override_fn_name = Some(Box::new(fun));
                    self
                }
            }
        }
    };

    // TODO: Make these rules recursive
    let do_check_dependency = match &injection_type {
        InjectionType::Arc { inner } => quote! { ::dill::OneOf::<#inner>::check(cat) },
        InjectionType::Reference { inner } => quote! { ::dill::OneOf::<#inner>::check(cat) },
        InjectionType::Option { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::Maybe::<::dill::OneOf::<#inner>>::check(cat) }
            }
            _ => unimplemented!("Currently only Option<Arc<Iface>> is supported"),
        },
        InjectionType::Lazy { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::specs::Lazy::<::dill::OneOf::<#inner>>::check(cat) }
            }
            _ => unimplemented!("Currently only Option<Arc<Iface>> is supported"),
        },
        InjectionType::Vec { item } => match item.as_ref() {
            InjectionType::Arc { inner } => quote! { ::dill::AllOf::<#inner>::check(cat) },
            _ => unimplemented!("Currently only Vec<Arc<Iface>> is supported"),
        },
        InjectionType::Value { typ } => quote! { ::dill::OneOf::<#typ>::check(cat) },
    };
    let check_dependency = match &injection_type {
        InjectionType::Reference { .. } => quote! { #do_check_dependency },
        _ => quote! {
            match &self.#override_fn_name {
                Some(_) => Ok(()),
                _ => #do_check_dependency,
            }
        },
    };

    let from_catalog = match &injection_type {
        InjectionType::Arc { inner } => quote! { ::dill::OneOf::<#inner>::get(cat)? },
        InjectionType::Reference { inner } => quote! { ::dill::OneOf::<#inner>::get(cat)? },
        InjectionType::Option { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::Maybe::<::dill::OneOf::<#inner>>::get(cat)? }
            }
            _ => unimplemented!("Currently only Option<Arc<Iface>> is supported"),
        },
        InjectionType::Lazy { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::specs::Lazy::<::dill::OneOf::<#inner>>::get(cat)? }
            }
            _ => unimplemented!("Currently only Lazy<Arc<Iface>> is supported"),
        },
        InjectionType::Vec { item } => match item.as_ref() {
            InjectionType::Arc { inner } => quote! { ::dill::AllOf::<#inner>::get(cat)? },
            _ => unimplemented!("Currently only Vec<Arc<Iface>> is supported"),
        },
        InjectionType::Value { typ } => {
            quote! { ::dill::OneOf::<#typ>::get(cat).map(|v| v.as_ref().clone())? }
        }
    };

    let prepare_dependency = match &injection_type {
        InjectionType::Reference { .. } => quote! { let #name = #from_catalog; },
        _ => quote! {
            let #name = match &self.#override_fn_name {
                Some(fun) => fun(cat)?,
                _ => #from_catalog,
            };
        },
    };

    let provide_dependency = match &injection_type {
        InjectionType::Reference { .. } => quote! { #name.as_ref() },
        _ => quote! { #name },
    };

    (
        override_fn_field,
        override_fn_field_ctor,
        override_setters,
        prepare_dependency,
        provide_dependency,
        check_dependency,
    )
}

/////////////////////////////////////////////////////////////////////////////////////////

fn implement_meta_var(index: usize, expr: &syn::ExprStruct) -> proc_macro2::TokenStream {
    let ident = format_ident!("_meta_{index}");
    let typ = &expr.path;
    quote! {
        const #ident: #typ = #expr;
    }
}

fn implement_meta_provide(index: usize, _expr: &syn::ExprStruct) -> proc_macro2::TokenStream {
    let ident = format_ident!("_meta_{index}");
    quote! {
        if !clb(&Self::#ident) { return }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Searches for `#[scope(X)]` attribute and returns `X`
fn get_scope(attrs: &Vec<syn::Attribute>) -> Option<syn::Path> {
    let mut scope = None;

    for attr in attrs {
        if is_dill_attr(attr, "scope") {
            attr.parse_nested_meta(|meta| {
                scope = Some(meta.path);
                Ok(())
            })
            .unwrap();
        }
    }

    scope
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Searches for all `#[interface(X)]` attributes and returns all types
fn get_interfaces(attrs: &Vec<syn::Attribute>) -> Vec<syn::Type> {
    let mut interfaces = Vec::new();

    for attr in attrs {
        if is_dill_attr(attr, "interface") {
            let iface = attr.parse_args().unwrap();
            interfaces.push(iface);
        }
    }

    interfaces
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Searches for all `#[meta(X)]` attributes and returns all expressions
fn get_meta(attrs: &Vec<syn::Attribute>) -> Vec<syn::ExprStruct> {
    let mut meta = Vec::new();

    for attr in attrs {
        if is_dill_attr(attr, "meta") {
            let expr = attr.parse_args().unwrap();
            meta.push(expr);
        }
    }

    meta
}

/////////////////////////////////////////////////////////////////////////////////////////

fn is_dill_attr<I: ?Sized>(attr: &syn::Attribute, ident: &I) -> bool
where
    syn::Ident: PartialEq<I>,
{
    if attr.path().is_ident(ident) {
        true
    } else {
        attr.path().segments.len() == 2
            && &attr.path().segments[0].ident == "dill"
            && attr.path().segments[1].ident == *ident
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Searches `impl` block for `new()` method
fn get_new(impl_items: &[syn::ImplItem]) -> Option<&syn::ImplItemFn> {
    impl_items
        .iter()
        .filter_map(|i| match i {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        })
        .find(|m| m.sig.ident == "new")
}
