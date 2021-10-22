use proc_macro::TokenStream;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::Field;
use syn::Token;

type StructFields = Punctuated<Field, Token![,]>;
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);

    match do_expand(&st) {
        Ok(tokenstream) => tokenstream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ret = generate_debug_trait(st)?;
    Ok(ret)
}

fn generate_debug_trait(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let associated_types_map = get_generic_associated_types(st);
    let struct_name_ident = &st.ident;

    let mut generics = st.generics.clone();

    let struct_name_literal = struct_name_ident.to_string();

    let mut fmt_body_stream = proc_macro2::TokenStream::new();

    fmt_body_stream.extend(quote::quote! {
       fmt.debug_struct(#struct_name_literal)
    });
    let mut field_type_names = Vec::new();
    let mut phantomdadta_type_param_names = Vec::new();
    let fields = get_fields_from_derive_input(&st)?;
    for field in fields.iter() {
        if let Some(s) = get_field_type_name(field)? {
            field_type_names.push(s);
        }
        if let Some(s) = get_phantomdata_generic_type_name(field)? {
            phantomdadta_type_param_names.push(s);
        }

        let field_name_ident = field.ident.as_ref().unwrap();

        let field_name_literal = field_name_ident.to_string();
        let mut format_str = "{:?}".to_string();

        if let Some(format) = get_custom_format_of_field(field)? {
            format_str = format;
        }

        fmt_body_stream.extend(quote::quote! {
            .field(#field_name_literal, &format_args!(#format_str,&self.#field_name_ident))
        });
    }

    fmt_body_stream.extend(quote::quote! {
        .finish()
    });

    if let Some(hatch) = get_struct_escape_hatch(st) {
        generics.make_where_clause();
        generics
            .where_clause
            .as_mut()
            .unwrap()
            .predicates
            .push(syn::parse_str(hatch.as_str()).unwrap());
    } else {
        for param in generics.params.iter_mut() {
            if let syn::GenericParam::Type(t) = param {
                let type_param_name = t.ident.to_string();

                if phantomdadta_type_param_names.contains(&type_param_name)
                    && !field_type_names.contains(&type_param_name)
                {
                    continue;
                }

                // 如果是关联类型，就不要对泛型参数`T`本身再添加约束了,除非`T`本身也被直接使用了
                if associated_types_map.contains_key(&type_param_name)
                    && !field_type_names.contains(&type_param_name)
                {
                    continue;
                }

                // 为泛型 T 添加限制
                t.bounds.push(syn::parse_quote!(std::fmt::Debug));
            }
        }
        generics.make_where_clause();
        // 关联类型的约束要放到where子句里
        for (_, associated_types) in associated_types_map {
            for associated_type in associated_types {
                generics
                    .where_clause
                    .as_mut()
                    .unwrap()
                    .predicates
                    .push(syn::parse_quote!(#associated_type:std::fmt::Debug));
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let ret_stream = quote::quote! {
        impl #impl_generics std::fmt::Debug for #struct_name_ident #ty_generics #where_clause{
            fn fmt(&self,fmt:&mut std::fmt::Formatter)->std::fmt::Result{
                #fmt_body_stream
            }
        }
    };

    Ok(ret_stream)
}

//获取字段从 deriveInput
fn get_fields_from_derive_input(st: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = st.data
    {
        return Ok(named);
    }
    Err(syn::Error::new_spanned(
        st,
        "Must define on a Struct,not Enum".to_string(),
    ))
}

fn get_custom_format_of_field(field: &syn::Field) -> syn::Result<Option<String>> {
    for attr in field.attrs.iter() {
        if let syn::Meta::NameValue(syn::MetaNameValue {
            ref path, ref lit, ..
        }) = attr.parse_meta()?
        {
            if path.is_ident("debug") {
                if let syn::Lit::Str(ref lit) = lit {
                    return Ok(Some(lit.value()));
                }
            } else {
                return Err(syn::Error::new_spanned(field, r#"expected `debug=...`"#));
            }
        }
    }

    Ok(None)
}

fn get_phantomdata_generic_type_name(f: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = f.ty
    {
        let ps = segments.first();

        if let Some(syn::PathSegment {
            ref ident,
            ref arguments,
        }) = ps
        {
            if ident.eq("PhantomData") {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = arguments
                {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                        ref path,
                        ..
                    }))) = args.first()
                    {
                        let ps = path.segments.first();
                        if let Some(syn::PathSegment { ref ident, .. }) = ps {
                            return Ok(Some(ident.to_string()));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn get_field_type_name(f: &syn::Field) -> syn::Result<Option<String>> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { ref segments, .. },
        ..
    }) = f.ty
    {
        if let Some(syn::PathSegment { ref ident, .. }) = segments.first() {
            return Ok(Some(ident.to_string()));
        }
    }
    Ok(None)
}

struct TypePathVisit {
    generic_type_names: Vec<String>,
    associated_types: HashMap<String, Vec<syn::TypePath>>,
}

impl<'ast> Visit<'ast> for TypePathVisit {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if node.path.segments.len() >= 2 {
            let generic_type_name = node.path.segments[0].ident.to_string();
            if self.generic_type_names.contains(&generic_type_name) {
                self.associated_types
                    .entry(generic_type_name)
                    .or_insert(Vec::new())
                    .push(node.clone())
            }
        }
        // Visit 模式要求在当前节点访问完成后，继续调用默认实现的visit方法，从而遍历到所有的
        // 必须调用这个函数，否则遍历到这个节点就不再往更深层走了
        visit::visit_type_path(self, node)
    }
}

fn get_generic_associated_types(st: &syn::DeriveInput) -> HashMap<String, Vec<syn::TypePath>> {
    // 找出泛型
    let origin_generic_params: Vec<String> = st
        .generics
        .params
        .iter()
        .filter_map(|f| {
            if let syn::GenericParam::Type(ty) = f {
                return Some(ty.ident.to_string());
            } else {
                None
            }
        })
        .collect();

    // 根据泛型去找是否关联的其他类型
    let mut visitor = TypePathVisit {
        generic_type_names: origin_generic_params,
        associated_types: HashMap::new(),
    };

    visitor.visit_derive_input(st);

    return visitor.associated_types;
}

fn get_struct_escape_hatch(st: &syn::DeriveInput) -> Option<String> {
    if let Some(attr) = st.attrs.last() {
        if let Ok(syn::Meta::List(syn::MetaList {
            ref path,
            ref nested,
            ..
        })) = attr.parse_meta()
        {
            if path.is_ident("debug") {
                if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    lit: syn::Lit::Str(ref lit),
                    ref path,
                    ..
                }))) = nested.first()
                {
                    if path.is_ident("bound") {
                        return Some(lit.value());
                    }
                }
            }
        }
    }

    None
}
