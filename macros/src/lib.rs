extern crate proc_macro;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn profile_zone (
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream { 
    let mut function = parse_macro_input!(item as ItemFn);
    let mut stmts: Vec<syn::Stmt> = function.block.stmts;
    let fn_name = function.sig.ident.to_string();

    let start_time_block: syn::Stmt = parse_quote! {
        {
            unsafe {
                use cpu_timer::*;
                TIMED_BLOCK.entry(#fn_name.to_string())
                .and_modify(|x| {
                    x.elapsed += read_cpu_timer() as usize - x.start;
                    x.count += 1;
                })
                .or_insert(TimedBlock { start: read_cpu_timer() as usize, elapsed: 0, count: 1 });
            }
        }
    };
    let end_time_block: syn::Stmt = parse_quote! {
        {
            unsafe {
                use cpu_timer::*;
                TIMED_BLOCK.entry(#fn_name.to_string())
                .and_modify(|x| x.elapsed += read_cpu_timer() as usize - x.start)
                .or_insert(TimedBlock { start: read_cpu_timer() as usize, elapsed: 0, count: 1 });
            }
        }
    };

    match function.sig.output {
        syn::ReturnType::Default => {
            let new_body: syn::Block = parse_quote! {
                {
                #start_time_block
                #(#stmts)*
                #end_time_block
                }
            };

            function.block = Box::new(new_body);
        },
        _ => {
            let ret_stmt = stmts.pop().unwrap();
            for s in &stmts {
                println!("s {:?}", s);
            }
            let new_body: syn::Block = parse_quote! {
                {
                #start_time_block
                #(#stmts)*
                #end_time_block
                #ret_stmt
                }
            };
            function.block = Box::new(new_body);
        },
    }

    return (quote!{#function}).into();
}
