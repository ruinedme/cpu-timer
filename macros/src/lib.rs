extern crate proc_macro;
//use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn profile(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let fn_name = function.sig.ident.to_string();

    let body: &syn::Block = &function.block;

    //WRAP SYNTAX IN { } OR YOUR LIFE WILL BE HELL
    let new_body: syn::Block = parse_quote! {
        {
        unsafe {
            use cpu_timer::*;
            TIMED_BLOCK.insert(#fn_name.to_string(),TimedBlock { start: read_cpu_timer() as usize, elapsed: 0, count: 1 });
        }
        #body
        unsafe {
            use cpu_timer::*;
            let block_end = read_cpu_timer() as usize;
            let cpu_freq = cpu_freq();
            let profile_block = TIMED_BLOCK.get_mut(#fn_name).unwrap();
            profile_block.elapsed += block_end - profile_block.start;
            
            //let total = block_end - profile_block.start;
    
            println!("Total {}: {:.4}ms", #fn_name, (profile_block.elapsed as f64 / cpu_freq as f64) * 1000f64 );
            let mut acc_total = 0;
            for block in &cpu_timer::TIMED_BLOCK {
                if block.0 == #fn_name {
                    continue;
                }
                acc_total += block.1.elapsed;
                println!("{} [{}] took: {}, {:.4}%", block.0, block.1.count, block.1.elapsed, (block.1.elapsed as f64 / profile_block.elapsed as f64) * 100f64);
            }
            let diff = profile_block.elapsed.abs_diff(acc_total);
            println!("profiled total: {} , diff: {}, {:.4}%", acc_total, diff, (acc_total as f64 / profile_block.elapsed as f64) * 100f64 );
        }

        }
    };

    function.block = Box::new(new_body);

    (quote! {
        #function
    })
    .into()
}
