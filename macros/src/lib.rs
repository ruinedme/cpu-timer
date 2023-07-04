extern crate proc_macro;
//use proc_macro::{TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn profile(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let fn_name = function.sig.ident.to_string();
    let fn_start = format_ident!("_{}_start", fn_name);
    let fn_end = format_ident!("_{}_end", fn_name);

    let body: &syn::Block = &function.block;

    //WRAP SYNTAX IN { } OR YOUR LIFE WILL BE HELL
    let new_body: syn::Block = parse_quote! {
        {
        let #fn_start: u64 = cpu_timer::read_cpu_timer();
        unsafe {
            cpu_timer::PROFILED_BLOCKS.push(cpu_timer::ProfiledBlocks {name: #fn_name, start: #fn_start, end: 0});
        }
        #body
        let #fn_end: u64 = cpu_timer::read_cpu_timer();
        unsafe {
            cpu_timer::PROFILED_BLOCKS[0].end = #fn_end;
            let total = cpu_timer::PROFILED_BLOCKS[0].end - cpu_timer::PROFILED_BLOCKS[0].start;
            println!("Total {}: {}", cpu_timer::PROFILED_BLOCKS[0].name, total);
            let mut acc_total = 0;
            for block in &cpu_timer::PROFILED_BLOCKS {
                let block_total = block.end - block.start;
                if block_total != total {
                    acc_total += block_total;
                    println!("{} took: {}, {:.4}%", block.name, block_total, (block_total as f64 / total as f64) * 100f64);
                }
            }
            let diff = total.abs_diff(acc_total);
            println!("profiled total: {} , diff: {}, {:.4}%", acc_total, diff, (acc_total as f64 / total as f64) * 100f64 );
        }

        }
    };

    function.block = Box::new(new_body);

    (quote! {
        #function
    })
    .into()
}
