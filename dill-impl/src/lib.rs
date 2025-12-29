extern crate proc_macro;

mod types;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use types::InjectionType;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct ComponentParams {
    vis: syn::Visibility,
    no_new: bool,
}

impl syn::parse::Parse for ComponentParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut params = ComponentParams {
            vis: syn::Visibility::Inherited,
            no_new: false,
        };

        while !input.is_empty() {
            if input.peek(syn::Token![pub]) {
                params.vis = input.parse()?;
            } else {
                let ident = input.parse::<syn::Ident>()?;
                match ident.to_string().as_str() {
                    "no_new" => params.no_new = true,
                    s => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("Unexpected parameter: {s}"),
                        ));
                    }
                }
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?; // Consume the comma
            }
        }
        Ok(params)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let params = syn::parse_macro_input!(attr as ComponentParams);

    let ast: syn::Item = syn::parse(item).unwrap();
    match ast {
        syn::Item::Struct(struct_ast) => component_from_struct(params, struct_ast),
        syn::Item::Impl(impl_ast) => component_from_impl(params, impl_ast),
        _ => {
            panic!("The #[component] macro can only be used on struct definition or an impl block")
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn scope(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn interface(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn meta(_args: TokenStream, item: TokenStream) -> TokenStream {
    item
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn component_from_struct(params: ComponentParams, mut ast: syn::ItemStruct) -> TokenStream {
    let impl_name = &ast.ident;
    let impl_type = syn::parse2(quote! { #impl_name }).unwrap();
    let impl_generics = syn::parse2(quote! {}).unwrap();

    let args: Vec<_> = ast
        .fields
        .iter_mut()
        .map(|f| {
            (
                f.ident.clone().unwrap(),
                f.ty.clone(),
                extract_attr_explicit(&mut f.attrs),
            )
        })
        .collect();

    let scope_type =
        get_scope(&ast.attrs).unwrap_or_else(|| syn::parse_str("::dill::Transient").unwrap());

    let interfaces = get_interfaces(&ast.attrs);
    let meta = get_meta(&ast.attrs);

    let mut stream: TokenStream = quote! { #ast }.into();

    if !params.no_new {
        stream.extend(implement_new(&impl_type, &args));
    }

    let builder: TokenStream = implement_builder(
        &ast.vis,
        &impl_type,
        &impl_generics,
        scope_type,
        interfaces,
        meta,
        args,
        !params.no_new,
    );

    stream.extend(builder);
    stream
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn component_from_impl(params: ComponentParams, mut ast: syn::ItemImpl) -> TokenStream {
    let impl_generics = &ast.generics;
    let impl_type = &ast.self_ty;
    let new = get_new(&mut ast.items).expect(
        "When using #[component] macro on the impl block it's expected to contain a new() \
         function. Otherwise use #[derive(Builder)] on the struct.",
    );

    let args: Vec<_> = new
        .sig
        .inputs
        .iter_mut()
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
                extract_attr_explicit(&mut arg.attrs),
            )
        })
        .collect();

    let scope_type =
        get_scope(&ast.attrs).unwrap_or_else(|| syn::parse_str("::dill::Transient").unwrap());

    let interfaces = get_interfaces(&ast.attrs);
    let meta = get_meta(&ast.attrs);

    let mut stream: TokenStream = quote! { #ast }.into();
    let builder: TokenStream = implement_builder(
        &params.vis,
        impl_type,
        impl_generics,
        scope_type,
        interfaces,
        meta,
        args,
        true,
    );

    stream.extend(builder);
    stream
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::too_many_arguments)]
fn implement_new(impl_type: &syn::Type, args: &[(syn::Ident, syn::Type, bool)]) -> TokenStream {
    let arg_decl = args.iter().map(|(name, ty, _)| quote! {#name: #ty});
    let arg_name = args.iter().map(|(name, _, _)| name);

    quote! {
        impl #impl_type {
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                #(#arg_decl),*
            ) -> Self {
                Self {
                    #(#arg_name),*
                }
            }
        }
    }
    .into()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::too_many_arguments)]
fn implement_builder(
    impl_vis: &syn::Visibility,
    impl_type: &syn::Type,
    _impl_generics: &syn::Generics,
    scope_type: syn::Path,
    interfaces: Vec<syn::Type>,
    meta: Vec<syn::ExprStruct>,
    args: Vec<(syn::Ident, syn::Type, bool)>,
    has_new: bool,
) -> TokenStream {
    let builder_name = format_ident!("{}Builder", quote! { #impl_type }.to_string());

    let arg_name: Vec<_> = args.iter().map(|(name, _, _)| name).collect();

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

    for (name, typ, is_explicit) in &args {
        let (
            override_fn_field,
            override_fn_field_ctor,
            override_setters,
            prepare_dependency,
            provide_dependency,
            check_dependency,
        ) = implement_arg(name, typ, &builder_name, *is_explicit);

        arg_override_fn_field.push(override_fn_field);
        arg_override_fn_field_ctor.push(override_fn_field_ctor);
        arg_override_setters.push(override_setters);
        arg_prepare_dependency.push(prepare_dependency);
        arg_provide_dependency.push(provide_dependency);
        arg_check_dependency.push(check_dependency);
    }

    let explicit_arg_decl: Vec<_> = args
        .iter()
        .filter(|(_, _, is_explicit)| *is_explicit)
        .map(|(ident, ty, _)| quote! { #ident: #ty })
        .collect();
    let explicit_arg_provide: Vec<_> = args
        .iter()
        .filter(|(_, _, is_explicit)| *is_explicit)
        .map(|(ident, _, _)| quote! { #ident })
        .collect();

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

    let component_or_explicit_factory = if explicit_arg_decl.is_empty() {
        quote! {
            impl ::dill::Component for #impl_type {
                type Impl = #impl_type;
                type Builder = #builder_name;

                fn builder() -> Self::Builder {
                    #builder_name::new()
                }
            }
        }
    } else {
        quote! {
            impl #impl_type {
                #[allow(clippy::too_many_arguments)]
                pub fn builder(
                    #(#explicit_arg_decl),*
                ) -> #builder_name {
                    #builder_name::new(
                        #(#explicit_arg_provide),*
                    )
                }
            }
        }
    };

    let builder = quote! {
        #impl_vis struct #builder_name {
            dill_builder_scope: #scope_type,
            #(#arg_override_fn_field),*
        }

        impl #builder_name {
            #( #meta_vars )*

            pub fn new(
                #(#explicit_arg_decl),*
            ) -> Self {
                Self {
                    dill_builder_scope: #scope_type::new(),
                    #(#arg_override_fn_field_ctor),*
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

            fn get_any(&self, cat: &::dill::Catalog) -> Result<::std::sync::Arc<dyn ::std::any::Any + Send + Sync>, ::dill::InjectionError> {
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

                let inst = self.dill_builder_scope.get_or_create(cat, || {
                    let inst = self.build(cat)?;
                    Ok(::std::sync::Arc::new(inst))
                })?;

                Ok(inst.downcast().unwrap())
            }

            fn bind_interfaces(&self, cat: &mut ::dill::CatalogBuilder) {
                #(
                    cat.bind::<#interfaces, #impl_type>();
                )*
            }
        }

        #(
            // Allows casting TypedBuilder<T> into TypedBuilder<dyn I> for all declared interfaces
            impl ::dill::TypedBuilderCast<#interfaces> for #builder_name
            {
                fn cast(self) -> impl ::dill::TypedBuilder<#interfaces> {
                    struct _B(#builder_name);

                    impl ::dill::Builder for _B {
                        fn instance_type_id(&self) -> ::std::any::TypeId {
                            self.0.instance_type_id()
                        }
                        fn instance_type_name(&self) -> &'static str {
                            self.0.instance_type_name()
                        }
                        fn interfaces(&self, clb: &mut dyn FnMut(&::dill::InterfaceDesc) -> bool) {
                            self.0.interfaces(clb)
                        }
                        fn metadata<'a>(&'a self, clb: &mut dyn FnMut(&'a dyn std::any::Any) -> bool) {
                            self.0.metadata(clb)
                        }
                        fn get_any(&self, cat: &::dill::Catalog) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, ::dill::InjectionError> {
                            self.0.get_any(cat)
                        }
                        fn check(&self, cat: &::dill::Catalog) -> Result<(), ::dill::ValidationError> {
                            self.0.check(cat)
                        }
                    }

                    impl ::dill::TypedBuilder<#interfaces> for _B {
                        fn get(&self, cat: &::dill::Catalog) -> Result<::std::sync::Arc<#interfaces>, ::dill::InjectionError> {
                            match self.0.get(cat) {
                                Ok(v) => Ok(v),
                                Err(e) => Err(e),
                            }
                        }

                        fn bind_interfaces(&self, cat: &mut ::dill::CatalogBuilder) {
                            self.0.bind_interfaces(cat);
                        }
                    }

                    _B(self)
                }
            }
        )*
    };

    quote! {
        #component_or_explicit_factory

        #builder
    }
    .into()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn implement_arg(
    name: &syn::Ident,
    typ: &syn::Type,
    builder: &syn::Ident,
    is_explicit: bool,
) -> (
    proc_macro2::TokenStream, // override_fn_field
    proc_macro2::TokenStream, // override_fn_field_ctor
    proc_macro2::TokenStream, // override_setters
    proc_macro2::TokenStream, // prepare_dependency
    proc_macro2::TokenStream, // provide_dependency
    proc_macro2::TokenStream, // check_dependency
) {
    let override_fn_name = format_ident!("arg_{}_fn", name);

    let injection_type = if is_explicit {
        InjectionType::Value { typ: typ.clone() }
    } else {
        types::deduce_injection_type(typ)
    };

    // Used to declare the field that stores the override factory function or
    // an explicit argument
    let override_fn_field = if is_explicit {
        quote! { #name: #typ }
    } else {
        match &injection_type {
            InjectionType::Reference { .. } => proc_macro2::TokenStream::new(),
            _ => quote! {
                #override_fn_name: Option<Box<dyn Fn(&::dill::Catalog) -> Result<#typ, ::dill::InjectionError> + Send + Sync>>
            },
        }
    };

    // Used initialize the field that stores the override factory function or
    // an explicit argument
    let override_fn_field_ctor = if is_explicit {
        quote! { #name: #name }
    } else {
        match &injection_type {
            InjectionType::Reference { .. } => proc_macro2::TokenStream::new(),
            _ => quote! { #override_fn_name: None },
        }
    };

    // Used to create with_* and with_*_fn setters for dependency overrides
    let override_setters = if is_explicit {
        proc_macro2::TokenStream::new()
    } else {
        match &injection_type {
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
        }
    };

    // Used in TBuilder::check() to validate the dependency
    let check_dependency = if is_explicit {
        quote! { Ok(()) }
    } else {
        let do_check_dependency = get_do_check_dependency(&injection_type);
        match &injection_type {
            InjectionType::Reference { .. } => quote! { #do_check_dependency },
            _ => quote! {
                match &self.#override_fn_name {
                    Some(_) => Ok(()),
                    _ => #do_check_dependency,
                }
            },
        }
    };

    // Used in TBuilder::build() to extract the dependency from the catalog
    let prepare_dependency = if is_explicit {
        proc_macro2::TokenStream::new()
    } else {
        let do_get_dependency = get_do_get_dependency(&injection_type);
        match &injection_type {
            InjectionType::Reference { .. } => quote! { let #name = #do_get_dependency; },
            _ => quote! {
                let #name = match &self.#override_fn_name {
                    Some(fun) => fun(cat)?,
                    _ => #do_get_dependency,
                };
            },
        }
    };

    // Called to provide dependency value to T's constructor
    let provide_dependency = if is_explicit {
        quote! { self.#name.clone() }
    } else {
        match &injection_type {
            InjectionType::Reference { .. } => quote! { #name.as_ref() },
            _ => quote! { #name },
        }
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_do_check_dependency(injection_type: &InjectionType) -> proc_macro2::TokenStream {
    match injection_type {
        InjectionType::Arc { inner } => quote! { ::dill::OneOf::<#inner>::check(cat) },
        InjectionType::Reference { inner } => quote! { ::dill::OneOf::<#inner>::check(cat) },
        InjectionType::Option { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::Maybe::<::dill::OneOf::<#inner>>::check(cat) }
            }
            InjectionType::Value { typ } => {
                quote! { ::dill::Maybe::<::dill::OneOf::<#typ>>::check(cat) }
            }
            _ => {
                unimplemented!("Currently only Option<Arc<Iface>> and Option<Value> are supported")
            }
        },
        InjectionType::Lazy { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::specs::Lazy::<::dill::OneOf::<#inner>>::check(cat) }
            }
            _ => unimplemented!("Currently only Lazy<Arc<Iface>> is supported"),
        },
        InjectionType::Vec { item } => match item.as_ref() {
            InjectionType::Arc { inner } => quote! { ::dill::AllOf::<#inner>::check(cat) },
            _ => unimplemented!("Currently only Vec<Arc<Iface>> is supported"),
        },
        InjectionType::Value { typ } => quote! { ::dill::OneOf::<#typ>::check(cat) },
    }
}

fn get_do_get_dependency(injection_type: &InjectionType) -> proc_macro2::TokenStream {
    match injection_type {
        InjectionType::Arc { inner } => quote! { ::dill::OneOf::<#inner>::get(cat)? },
        InjectionType::Reference { inner } => quote! { ::dill::OneOf::<#inner>::get(cat)? },
        InjectionType::Option { element } => match element.as_ref() {
            InjectionType::Arc { inner } => {
                quote! { ::dill::Maybe::<::dill::OneOf::<#inner>>::get(cat)? }
            }
            InjectionType::Value { typ } => {
                quote! { ::dill::Maybe::<::dill::OneOf::<#typ>>::get(cat)?.map(|v| v.as_ref().clone()) }
            }
            _ => {
                unimplemented!("Currently only Option<Arc<Iface>> and Option<Value> are supported")
            }
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
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Searches for `#[scope(X)]` attribute and returns `X`
fn get_scope(attrs: &Vec<syn::Attribute>) -> Option<syn::Path> {
    let mut scope = None;

    for attr in attrs {
        if is_dill_attr(attr, "scope") {
            attr.parse_nested_meta(|meta| {
                scope = Some(meta.path);
                Ok(())
            })
            .expect("Could not parse scope");
        }
    }

    scope
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Searches `impl` block for `new()` method
fn get_new(impl_items: &mut [syn::ImplItem]) -> Option<&mut syn::ImplItemFn> {
    impl_items
        .iter_mut()
        .filter_map(|i| match i {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        })
        .find(|m| m.sig.ident == "new")
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn extract_attr_explicit(attrs: &mut Vec<syn::Attribute>) -> bool {
    let mut present = false;
    attrs.retain_mut(|attr| {
        if is_attr_explicit(attr) {
            present = true;
            false
        } else {
            true
        }
    });
    present
}

fn is_attr_explicit(attr: &syn::Attribute) -> bool {
    if !is_dill_attr(attr, "component") {
        return false;
    }
    let syn::Meta::List(meta) = &attr.meta else {
        return false;
    };
    meta.tokens.to_string().contains("explicit")
}
