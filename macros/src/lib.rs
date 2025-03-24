use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    quote! {
        #[cfg_attr(target_arch = "riscv32", no_mangle)]
        #[allow(unused)]
        #func
    }
    .into()
}
