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
    let function_block = input.block;
    let mut function_first = quote! {};
    let mut function_enter = quote! {};
    let mut function_leave = quote! { ValueType::Null };

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

            let block_name = block_name.expect("no argument name");
            let block_type = block_type.expect("no argument type");

            match block_type.to_string().as_str() {
                "String" => {
                    function_enter = quote! {
                        #function_enter
                        ValueType::String,
                    };
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_string();
                    }
                }
                "i32" => {
                    function_enter = quote! {
                        #function_enter
                        ValueType::Integer,
                    };
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_integer() as i32;
                    }
                }
                "i64" => {
                    function_enter = quote! {
                        #function_enter
                        ValueType::Integer,
                    };
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_integer();
                    }
                }
                "f32" => {
                    function_enter = quote! {
                        #function_enter
                        ValueType::Decimal,
                    };
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_decimal() as f32;
                    }
                }
                "f64" => {
                    function_enter = quote! {
                        #function_enter
                        ValueType::Decimal,
                    };
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_decimal();
                    }
                }
                "bool" => {
                    function_enter = quote! {
                        #function_enter
                        ValueType::Boolean,
                    };
                    function_first = quote! {
                        #function_first
                        let #block_name = argument.next().unwrap().as_boolean();
                    }
                }
                _ => {}
            }
        }
    }

    if let syn::ReturnType::Type(_, kind) = input.sig.output
        && let syn::Type::Path(kind) = &*kind
    {
        let kind = kind.path.get_ident().expect("no return type").to_string();

        match kind.as_str() {
            "String" => {
                function_leave = quote! { ValueType::String };
            }
            "i32" => {
                function_leave = quote! { ValueType::Integer };
            }
            "i64" => {
                function_leave = quote! { ValueType::Integer };
            }
            "f32" => {
                function_leave = quote! { ValueType::Decimal };
            }
            "f64" => {
                function_leave = quote! { ValueType::Decimal };
            }
            "bool" => {
                function_leave = quote! { ValueType::Boolean };
            }
            _ => {}
        }
    }

    TokenStream::from(quote! {
        const #name_metadata : FunctionMeta = FunctionMeta::new(
            NativeArgument::Constant(&[
                #function_enter
            ]),
            #function_leave
        );
        fn #name_function(machine: &mut Machine, mut argument: Argument) -> Option<Value> {
            #function_first
            #function_block
        }
    })
}

#[proc_macro]
pub fn function_add(input: TokenStream) -> TokenStream {
    let mut scope_name = None;
    let mut function_name = None;
    let mut function_meta = None;

    let argument = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);

    for expression in argument {
        if let Expr::Path(path) = expression {
            let path = path.path.get_ident().cloned();

            if scope_name.is_none() {
                scope_name = path;
            } else if function_name.is_none() {
                function_name = path.clone();
                function_meta = Some(format_ident!(
                    "alicia_meta_{}",
                    path.clone().unwrap().to_string()
                ));
            }
        }
    }

    let scope_name = scope_name.expect("no name for scope");
    let function_name = function_name.expect("no function name");
    let function_path = function_name.clone();
    let function_name = function_name.to_string();
    let function_meta = function_meta.expect("no function meta");

    /*
    scope.symbol.insert({name}, Declaration::FunctionNative(FunctionNative {
        name: {name},
        call: {path},
        enter: Self::alicia_meta_{name}.enter,
        leave: Self::alicia_meta_{name}.leave,
    }));
    */

    TokenStream::from(quote! {
        {
            let _ = #scope_name.symbol.insert(#function_name.to_string(), Declaration::FunctionNative(FunctionNative {
                name: #function_name.to_string(),
                call: Self::#function_path,
                enter: Self::#function_meta.enter,
                leave: Self::#function_meta.leave,
            }));
        }
    })
}

#[proc_macro]
pub fn builder_function(input: TokenStream) -> TokenStream {
    let mut function_name = None;
    let mut function_meta = None;

    let argument = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);

    for expression in argument {
        if let Expr::Path(path) = expression {
            let path = path.path.get_ident().cloned();

            if function_name.is_none() {
                function_name = path.clone();
                function_meta = Some(format_ident!(
                    "alicia_meta_{}",
                    path.clone().unwrap().to_string()
                ));
            }
        }
    }

    let function_name = function_name.expect("no function name");
    let function_path = function_name.clone();
    let function_name = function_name.to_string();
    let function_meta = function_meta.expect("no function meta");

    /*
    FunctionNative::new(
        #function_name.to_string(),
        self::#function_path,
        self::#function_meta.enter,
        self::#function_meta.leave,
    )
    */

    TokenStream::from(quote! {
        FunctionNative::new(
            #function_name.to_string(),
            self::#function_path,
            self::#function_meta.enter,
            self::#function_meta.leave,
        )
    })
}
