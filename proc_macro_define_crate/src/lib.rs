use proc_macro::TokenStream;

use quote::quote;
use syn::parse_macro_input::parse;
use syn::{parse_macro_input, Attribute, AttributeArgs, Item, Meta, MetaList, NestedMeta, Path};

#[proc_macro_attribute]
pub fn my_test_proc_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse::<AttributeArgs>(attr);

    let attr = match a {
        Ok(a) => a,
        Err(err) => {
            return TokenStream::from(err.to_compile_error());
        }
    };

    // let attr = parse_macro_input!(attr as AttributeArgs);

    for i in attr.iter() {
        if let NestedMeta::Meta(Meta::List(MetaList {
            path: Path { segments, .. },
            ..
        })) = i
        {
            for s in segments.iter() {
                eprintln!("{:#?}", s.ident);
            }
        }
    }
    let item = parse_macro_input!(item as Item);
    eprintln!("attribute:\n{:#?}", attr);
    eprintln!("item:\n{:#?}", item);
    quote!(#item).into()
}
