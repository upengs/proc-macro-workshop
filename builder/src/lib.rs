use proc_macro::TokenStream;
use std::option::Option::Some;
use syn::spanned::Spanned;

/// 派生式的过程宏
/// 如何向编译器注册一个惰性属性的名字呢
/// 简单来说，就是要在#[proc_macro_derive(Builder)]
/// 这个标签中添加上属性的名字，加入我们要加入一个名为builder的属性
/// 那么就要这样写：
/// #[proc_macro_derive(Builder, attributes(builder))]
///  
///
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    // 解析tokenstream 为DeriveInput
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    eprintln!("{:#?}", &st);
    match do_expand(&st) {
        Ok(tokenstream) => tokenstream,
        Err(e) => e.to_compile_error().into(),
    }
}

type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token!(,)>;

fn get_fields_from_derive_input(st: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = &st.data
    {
        return Ok(named);
    }

    Err(syn::Error::new_spanned(
        st,
        "Must define on a Struct, not Enum".to_string(),
    ))
}

fn generate_builder_struct_fields_def(
    fields: &StructFields,
) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    let types: syn::Result<Vec<proc_macro2::TokenStream>> = fields
        .iter()
        .map(|f| {
            if let Some(inner_ty) = get_generic_inner_type(&f.ty, "Option") {
                Ok(quote::quote! (std::option::Option<#inner_ty>))
            } else if get_user_specified_attribute_ident_for_vec(f)?.is_some() {
                let origin_ty = &f.ty;
                Ok(quote::quote! (#origin_ty))
            } else {
                let inner_ty = &f.ty;
                Ok(quote::quote! (std::option::Option<#inner_ty>))
            }
        })
        .collect();
    let types = types?;
    let token_stream = quote::quote! {
        #(#idents : #types,)*
    };
    Ok(token_stream)
}

fn generate_builder_struct_factory_init_clauses(
    fields: &StructFields,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let init_clauses: syn::Result<Vec<proc_macro2::TokenStream>> = fields
        .iter()
        .map(|f| {
            let ident = &f.ident;

            if get_user_specified_attribute_ident_for_vec(f)?.is_some() {
                Ok(quote::quote! {
                    #ident : std::vec::Vec::new(),
                })
            } else {
                Ok(quote::quote! {
                    #ident : std::option::Option::None,
                })
            }
        })
        .collect();

    Ok(init_clauses?)
}

fn generate_setter_functions(fields: &StructFields) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let mut final_tokenstream = proc_macro2::TokenStream::new();

    for (idx, (ident, type_)) in idents.iter().zip(types.iter()).enumerate() {
        let mut tokenstream_piece;
        if let Some(inner_ty) = get_generic_inner_type(type_, "Option") {
            tokenstream_piece = quote::quote! {
                fn #ident(&mut self,#ident : #inner_ty)->&mut Self{
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            };
        } else if let Some(ref use_specified_ident) =
            get_user_specified_attribute_ident_for_vec(&fields[idx])?
        {
            let inner_ty = get_generic_inner_type(type_, "Vec").ok_or(syn::Error::new(
                fields[idx].span(),
                "each field must be specified with Vec field",
            ))?;

            tokenstream_piece = quote::quote! {
                fn #use_specified_ident(&mut self, #use_specified_ident : #inner_ty)->&mut Self{
                    self.#ident.push(#use_specified_ident);
                    self
                }
            };

            if use_specified_ident != ident.as_ref().unwrap() {
                tokenstream_piece.extend(quote::quote! {
                fn #ident(&mut self,#ident:#type_)->&mut Self{
                   self.#ident = #ident.clone();
                   self
                       }
                   });
            }
        } else {
            tokenstream_piece = quote::quote! {
                fn #ident(&mut self,#ident : #type_)->&mut Self{
                    self.#ident = std::option::Option::Some(#ident);
                    self
                }
            };
        }
        // 不断追加新的TokenStream片段到一个公共的TokenStream上
        final_tokenstream.extend(tokenstream_piece);
    }
    Ok(final_tokenstream)
}

fn generate_build_function(
    fields: &StructFields,
    origin_struct_ident: &syn::Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    let types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let mut checker_code_pieces = Vec::new();
    let mut fill_result_clauses = Vec::new();

    for idx in 0..idents.len() {
        let ident = idents[idx];

        if get_generic_inner_type(types[idx], "Option").is_none()
            && get_user_specified_attribute_ident_for_vec(&fields[idx])?.is_none()
        {
            checker_code_pieces.push(quote::quote! {
                if self.#ident.is_none(){
                    let err = format!("{} field missing",stringify!(#ident));
                    return std::result::Result::Err(err.into());
                }
            });
        }

        if get_user_specified_attribute_ident_for_vec(&fields[idx])?.is_some() {
            fill_result_clauses.push(quote::quote! {
                #ident : self.#ident.clone(),
            });
        } else if get_generic_inner_type(types[idx], "Option").is_none() {
            fill_result_clauses.push(quote::quote! {
                #ident : self.#ident.clone().unwrap(),
            });
        } else {
            fill_result_clauses.push(quote::quote! {
                #ident : self.#ident.clone(),
            });
        }
    }

    let token_stream = quote::quote! {
        pub fn build(&mut self)->std::result::Result<#origin_struct_ident,std::boxed::Box<dyn std::error::Error>>{
             #(#checker_code_pieces)*

            let ret = #origin_struct_ident {
                #(#fill_result_clauses)*
            };

            std::result::Result::Ok(ret)
        }
    };

    Ok(token_stream)
}

// 获取用户指定的惰性属性的值
fn get_user_specified_attribute_ident_for_vec(
    field: &syn::Field,
) -> syn::Result<Option<syn::Ident>> {
    for attr in &field.attrs {
        if let Ok(syn::Meta::List(syn::MetaList {
            ref path,
            ref nested,
            ..
        })) = attr.parse_meta()
        {
            if let Some(p) = path.segments.first() {
                if p.ident == "builder" {
                    if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(kv))) = nested.first() {
                        if kv.path.is_ident("each") {
                            if let syn::Lit::Str(ref ident_str) = kv.lit {
                                return Ok(Some(syn::Ident::new(
                                    ident_str.value().as_str(),
                                    attr.span(),
                                )));
                            }
                        } else {
                            if let Ok(syn::Meta::List(ref list)) = attr.parse_meta() {
                                return Err(syn::Error::new_spanned(
                                    list,
                                    r#"expected `builder(each = "...")`"#,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// outer_ident_name  通过ident来判断最外面的类型
fn get_generic_inner_type<'a>(t: &'a syn::Type, outer_ident_name: &str) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = t {
        if let Some(seq) = path.segments.last() {
            if seq.ident == outer_ident_name {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = seq.arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

// 生成最终的TokenStream
fn do_expand(st: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let struct_name_literal = st.ident.to_string();
    let builder_name_literal = format!("{}Builder", struct_name_literal);

    // 构建一个新的标志符
    let builder_name_ident = syn::Ident::new(&builder_name_literal, st.span());

    // 获取派生中结构体Command struct fields
    let fields = get_fields_from_derive_input(&st)?;
    // 获取派生中结构体Command的属性字段
    let builder_struct_fields_def = generate_builder_struct_fields_def(fields)?;
    // 初始化CommandBuilder的新实例字段
    let init_clauses = generate_builder_struct_factory_init_clauses(fields)?;
    // 生成setter方法
    let setter_functions = generate_setter_functions(fields)?;

    let struct_ident = &st.ident;
    let build_function = generate_build_function(&fields, struct_ident)?;
    /// quote::quote!{}
    /// 可以将语法树节点及其子节点重新转化为proc_macro2::TokenStream
    /// 但是可以通过into()转换为proc_macro::TokenStream
    let expand = quote::quote! {
        pub struct #builder_name_ident{
            #builder_struct_fields_def
        }

        impl #builder_name_ident {
            #setter_functions
            #build_function
        }

        impl #struct_ident {
            pub fn builder()-> #builder_name_ident {
                 #builder_name_ident{
                    #(#init_clauses)*
                }
            }
        }
    };

    Ok(expand.into())
}
