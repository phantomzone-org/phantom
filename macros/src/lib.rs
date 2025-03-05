use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Expr, ExprMacro, ItemFn, Pat, PatTuple, Type};

// Note: This function is created with AI prompts and is not thoroughly reviewed
#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    // Count and validate macro usages
    let mut read_input_count = 0;
    let mut output_count = 0;

    for stmt in &func.block.stmts {
        match stmt {
            syn::Stmt::Local(local) if is_read_input_macro(local) => {
                read_input_count += 1;
            }
            syn::Stmt::Macro(macro_stmt) if is_output_macro(macro_stmt) => {
                output_count += 1;
            }
            _ => {}
        }
    }

    // Validate macro usage
    if read_input_count != 1 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Entry function must contain exactly one read_input!() macro call"),
        )
        .to_compile_error()
        .into();
    }

    if output_count > 1 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Entry function must contain no mre than one output!() macro call",
        )
        .to_compile_error()
        .into();
    }

    // Find the read_input!() macro call and its bindings
    let (var_names, var_types) = func
        .block
        .stmts
        .iter()
        .find_map(|stmt| {
            if let syn::Stmt::Local(local) = stmt {
                if let Pat::Type(pat_type) = &local.pat {
                    if let Pat::Tuple(PatTuple { elems, .. }) = &*pat_type.pat {
                        if let Some(init) = &local.init {
                            if let Expr::Macro(ExprMacro { mac, .. }) = &*init.expr {
                                if mac.path.segments.last().unwrap().ident == "read_input" {
                                    // Extract variable names from the pattern
                                    let var_names: Vec<_> = elems
                                        .iter()
                                        .map(|elem| {
                                            if let Pat::Ident(pat_ident) = elem {
                                                (pat_ident.ident.to_string(), pat_ident.mutability)
                                            } else {
                                                panic!("Unsupported pattern in tuple")
                                            }
                                        })
                                        .collect();

                                    // Extract types from the type annotation
                                    if let Type::Tuple(tuple_type) = &*pat_type.ty {
                                        let types: Vec<_> = tuple_type.elems.iter().collect();
                                        return Some((var_names, types));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        })
        .expect("read_input!() statement is missing");

    // Generate static arrays and deserialization code
    let mut static_declarations = quote!();
    let mut deserialize_statements = quote!();

    for ((var_ident, var_mut), var_type) in var_names.iter().zip(var_types.iter()) {
        let static_ident = syn::Ident::new(&var_ident.to_string().to_uppercase(), var_ident.span());

        static_declarations.extend(quote! {
            #[no_mangle]
            #[link_section = ".inpdata"]
            pub static #static_ident: [u8; core::mem::size_of::<#var_type>()] = [0; core::mem::size_of::<#var_type>()];
        });

        let var_ident = syn::Ident::new(&var_ident, var_ident.span());
        deserialize_statements.extend(quote! {
            let #var_mut #var_ident: #var_type = postcard::from_bytes(black_box(&#static_ident)).unwrap();
        });
    }

    // Transform statements, replacing both read_input!() and output!() macro calls
    let transformed_stmts = func.block.stmts.iter().filter_map(|stmt| match stmt {
        syn::Stmt::Local(local) if is_read_input_macro(local) => None,
        syn::Stmt::Macro(macro_stmt) if is_output_macro(macro_stmt) => {
            let expr = macro_stmt.mac.tokens.clone();
            Some(quote! {
                let output_des = postcard::to_allocvec(&#expr).unwrap();
                runtime::produce_output(&output_des);
            })
        }
        _ => Some(quote!(#stmt)),
    });

    let expanded = quote! {
        #static_declarations

        #[cfg_attr(target_arch = "riscv32", no_mangle)]
        #[allow(unused)]
        fn main() {
            use core::hint::black_box;

            #deserialize_statements

            #(#transformed_stmts)*
        }
    };

    expanded.into()
}

fn is_read_input_macro(local: &syn::Local) -> bool {
    if let Some(init) = &local.init {
        if let Expr::Macro(ExprMacro { mac, .. }) = &*init.expr {
            return mac
                .path
                .segments
                .last()
                .map_or(false, |seg| seg.ident == "read_input");
        }
    }
    false
}

fn is_output_macro(macro_stmt: &syn::StmtMacro) -> bool {
    macro_stmt
        .mac
        .path
        .segments
        .last()
        .map_or(false, |seg| seg.ident == "output")
}
