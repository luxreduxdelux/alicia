use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Expr;
use syn::Ident;
use syn::ItemFn;
use syn::Token;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn function(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemFn);

    let name_function = input.sig.ident.clone();
    let name_metadata = format_ident!("alicia_meta_{}", name_function.to_string());
    let mut function_first = quote! {};
    let function_block = input.block;

    for parameter in &input.sig.inputs {
        if let syn::FnArg::Typed(kind) = parameter {
            let mut block_name = None;
            let mut block_type = None;

            if let syn::Pat::Ident(pat_ident) = &*kind.pat {
                block_name = Some(pat_ident.ident.clone());
            }

            if let syn::Type::Path(type_path) = &*kind.ty {
                block_type = type_path.path.get_ident();
            }

            let block_name = block_name.unwrap();
            let block_type = block_type.unwrap();

            match block_type.to_string().as_str() {
                "String" => {
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_string();
                    }
                }
                "i32" => {
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_integer() as i32;
                    }
                }
                "i64" => {
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_integer();
                    }
                }
                "f32" => {
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_decimal() as f32;
                    }
                }
                "f64" => {
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_decimal();
                    }
                }
                "bool" => {
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_boolean();
                    }
                }
                _ => {}
            }
        }
    }

    TokenStream::from(quote! {
        const #name_metadata = FunctionMeta::new(
            NativeArgument::Variable,
            ValueType::Null
        );
        fn #name_function(machine: &mut Machine, mut argument: Argument) -> Option<Value> {
            #function_first
            #function_block
        }
    })
}

#[proc_macro]
pub fn function_add(input: TokenStream) -> TokenStream {
    input
}
