use {
    proc_macro2::Ident,
    proc_macro::TokenStream,
    proc_macro_error::{abort, abort_call_site, proc_macro_error, ResultExt},
    quote::quote,
    std::iter::repeat,
    syn::{
        Attribute,
        Data,
        DataStruct,
        DeriveInput,
        Field,
        LifetimeDef,
        parse_macro_input,
        spanned::Spanned,
        Type,
        TypeParam,
    },
};

#[proc_macro_derive(Builder)]
#[proc_macro_error]
pub fn derive_builder(input: TokenStream) -> TokenStream
{
    let ast = parse_macro_input!(input as DeriveInput);
    let DeriveInput {
        ident,
        generics,
        data,
        vis,
        ..
    } = ast;

    let fields = match data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => abort_call_site!("#[derive(Builder)] is only supported for structs")
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut lifetimes = generics.lifetimes()
        .map(|LifetimeDef { lifetime, .. }| lifetime)
        .peekable();
    let lifetimes = if lifetimes.peek().is_some() {
        quote! { #(#lifetimes),*, }
    } else {
        quote! {}
    };

    let mut impl_lifetimes = generics.lifetimes()
        .map(
            |LifetimeDef { lifetime, bounds, .. }| {
                if bounds.is_empty() {
                    quote! { #lifetime }
                } else {
                    quote! { #lifetime: #bounds }
                }
            }
        )
        .peekable();
    let impl_lifetimes = if impl_lifetimes.peek().is_some() {
        quote! { #(#impl_lifetimes),*, }
    } else {
        quote! {}
    };

    let mut type_params = generics.type_params().peekable();
    let type_params = if type_params.peek().is_some() {
        quote! { #(#type_params),*, }
    } else {
        quote! {}
    };

    struct FieldInfo {
        doc_attrs: Vec<Attribute>,
        ident: Ident,
        ty: Type,
    }

    let fields_to_init = fields.into_iter()
        .map(
            |field| {
                let field_span = field.span();
                let Field { attrs, ident, ty, .. } = field;
                let ident = ident.unwrap_or_else(
                    || abort!(
                        field_span,
                        "#[derive(Builder)]: only named fields are currently supported"
                    )
                );
                let doc_attrs = attrs.iter()
                    .filter(
                        |v| {
                            v.parse_meta()
                                .map(|meta| meta.path().is_ident("doc"))
                                .unwrap_or(false)
                        }
                    )
                    .cloned()
                    .collect();
                FieldInfo {
                    doc_attrs,
                    ident,
                    ty,
                }
            }
        )
        .collect::<Vec<_>>();

    let builder_name = format!("{ident}Builder");
    let builder_name = syn::parse_str::<Ident>(&builder_name)
        .map_err(|e| syn::Error::new(ident.span(), e))
        .expect_or_abort("#[derive(Builder)]: corresponding builder name is not an ident");

    fn generate_init_indicator(ident: &Ident) -> Ident
    {
        let const_generic_parameter = format!("{ident}_INIT");
        syn::parse_str::<Ident>(&const_generic_parameter)
            .map_err(|e| syn::Error::new(ident.span(), e))
            .expect_or_abort(
                "#[derive(Builder)]: can't generate corresponding const-generic indicator"
            )
    }

    let mut builder_consts = fields_to_init.iter()
        .map(|FieldInfo { ident, .. }| ident)
        .map(generate_init_indicator)
        .map(|ind| quote! { const #ind: bool })
        .peekable();
    let builder_consts = if builder_consts.peek().is_some() {
        quote! { #(#builder_consts),*, }
    } else {
        quote! {}
    };
    let builder_body = fields_to_init.iter()
        .map(
            |FieldInfo { ident, ty, .. }| quote! {
                #ident: ::std::mem::MaybeUninit<#ty>
            }
        );

    let builder_body = quote! {
        #vis struct #builder_name <#impl_lifetimes #builder_consts #type_params>
            #where_clause
        {
            #(#builder_body),*
        }
    };


    let mut all_consts_true = repeat(quote! { true })
        .take(fields_to_init.len())
        .peekable();
    let all_consts_true = if all_consts_true.peek().is_some() {
        quote! { #(#all_consts_true),*, }
    } else {
        quote! {}
    };
    let type_idents = generics.type_params()
        .map(|TypeParam { ident, .. }| ident)
        .collect::<Vec<_>>();
    let fields = fields_to_init.iter()
        .map(|FieldInfo { ident, .. }| ident);
    let fields_assume_init = fields_to_init.iter()
        .map(|FieldInfo { ident, .. }| quote! { #ident: #ident.assume_init() });

    let build_impl = quote! {
        impl #impl_generics #builder_name <#lifetimes #all_consts_true #(#type_idents),*>
            #where_clause
        {
            pub const fn build(self) -> #ident #ty_generics
            {
                let Self {
                    #(#fields),*
                } = self;
                unsafe {
                    #ident {
                        #(#fields_assume_init),*
                    }
                }
            }
        }
    };


    let mut all_consts_false = repeat(quote! { false })
        .take(fields_to_init.len())
        .peekable();
    let all_consts_false = if all_consts_false.peek().is_some() {
        quote! { #(#all_consts_false),*, }
    } else {
        quote! {}
    };
    let fields_uninit = fields_to_init.iter()
        .map(|FieldInfo { ident, .. }| quote! { #ident: ::std::mem::MaybeUninit::uninit() });

    let new_impl = quote! {
        impl #impl_generics #builder_name <#lifetimes #all_consts_false #(#type_idents),*>
            #where_clause
        {
            pub const fn new(self) -> Self
            {
                Self {
                    #(#fields_uninit),*
                }
            }
        }
    };

    let methods = fields_to_init.iter()
        .enumerate()
        .map(
            |(cur_idx, FieldInfo { doc_attrs, ident, ty })| {
                let mut impl_consts = fields_to_init.iter()
                    .map(|FieldInfo { ident, .. }| ident)
                    .enumerate()
                    .filter_map(
                        |(idx, field)| if idx != cur_idx {
                            Some(field)
                        } else {
                            None
                        }
                    )
                    .map(generate_init_indicator)
                    .map(|ind| quote! { const #ind: bool })
                    .peekable();
                let impl_consts = if impl_consts.peek().is_some() {
                    quote! { #(#impl_consts),*, }
                } else {
                    quote! {}
                };

                let mut consts_self = fields_to_init.iter()
                    .map(|FieldInfo { ident, .. }| ident)
                    .enumerate()
                    .map(
                        |(idx, field)| if idx != cur_idx {
                            let ind = generate_init_indicator(field);
                            quote! { #ind }
                        } else {
                            quote! { false }
                        }
                    )
                    .peekable();
                let consts_self = if consts_self.peek().is_some() {
                    quote! { #(#consts_self),*, }
                } else {
                    quote! {}
                };

                let mut consts_res = fields_to_init.iter()
                    .map(|FieldInfo { ident, .. }| ident)
                    .enumerate()
                    .map(
                        |(idx, field)| if idx != cur_idx {
                            let ind = generate_init_indicator(field);
                            quote! { #ind }
                        } else {
                            quote! { true }
                        }
                    )
                    .peekable();
                let consts_res = if consts_res.peek().is_some() {
                    quote! { #(#consts_res),*, }
                } else {
                    quote! {}
                };

                let mut fields = fields_to_init.iter()
                    .map(|FieldInfo { ident, .. }| ident)
                    .enumerate()
                    .filter_map(
                        |(idx, value)| if idx != cur_idx {
                            Some(value)
                        } else {
                            None
                        }
                    )
                    .peekable();
                let fields = if fields.peek().is_some() {
                    quote! { #(#fields),*, }
                } else {
                    quote! {}
                };

                quote! {
                    impl<#impl_lifetimes #impl_consts #type_params>
                    #builder_name <#lifetimes #consts_self #(#type_idents),*>
                        #where_clause
                    {
                        #(#doc_attrs)*
                        pub fn #ident(
                            self,
                            value: #ty) -> #builder_name <#lifetimes #consts_res #(#type_idents),*>
                        {
                            let Self {
                                #fields
                                ..
                            } = self;
                            #builder_name {
                                #fields
                                #ident: ::std::mem::MaybeUninit::new(value),
                            }
                        }
                    }
                }
            }
        );

    let tokens = quote! {
        #builder_body
        #build_impl
        #new_impl
        #(#methods)*
    };
    tokens.into()
}