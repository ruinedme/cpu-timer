extern  crate proc_macro;
//use proc_macro::{TokenStream};
use syn::{parse_macro_input, ItemFn, parse_quote};
use quote::{quote, format_ident};

#[proc_macro_attribute]
pub fn profile(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let fn_name = function.sig.ident.to_string();
    let fn_start = format_ident!("_{}_start", fn_name);
    let fn_end = format_ident!("_{}_end", fn_name);

    let body: &syn::Block = &function.block;

    //WRAP SYNTAX IN { } OR YOUR LIFE WILL BE HELL
    let new_body: syn::Block = parse_quote! {
        {
        let #fn_start: u64 = cpu_timer::read_cpu_timer();
        #body
        let #fn_end: u64 = cpu_timer::read_cpu_timer();
        println!("{} took: {}", #fn_name, #fn_end - #fn_start);
        
        }
    };

    function.block = Box::new(new_body);

    (quote! {
        #function
    }).into()
}
