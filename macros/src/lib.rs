extern crate proc_macro;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Block, Expr, ItemFn, Lit, MetaNameValue, ReturnType, Stmt,
};

#[proc_macro_attribute]
pub fn profile_zone(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = parse_macro_input!(attr as MetaNameValue);
    let init = if let Expr::Lit(base) = attrs.value {
        match base.lit {
            Lit::Bool(x) => x.value,
            _ => false,
        }
    } else {
        false
    };

    let mut function = parse_macro_input!(item as ItemFn);
    let mut stmts: Vec<Stmt> = function.block.stmts;
    let fn_name = function.sig.ident.to_string();

    let mut init_block: Stmt = parse_quote!({});
    if init {
        init_block = parse_quote! {
            {
                unsafe {
                    use cpu_timer::{PROFILER,read_cpu_timer};
                    PROFILER.start_tsc = read_cpu_timer();
                }
            }
        };
    }

    let start_time_block: Stmt = parse_quote! {
        {
            unsafe {
                use cpu_timer::{PROFILER,ProfileAnchor,read_cpu_timer};
                PROFILER.anchors
                .entry(#fn_name.to_string())
                .and_modify(|x| {
                    x.tsc_elapsed += read_cpu_timer() - x.start_tsc;
                    x.hit_count += 1;
                })
                .or_insert(ProfileAnchor { start_tsc: read_cpu_timer(), tsc_elapsed: 0, hit_count: 1 });
            }
        }
    };
    let end_time_block: Stmt = parse_quote! {
        {
            unsafe {
                use cpu_timer::{PROFILER,ProfileAnchor,read_cpu_timer};
                PROFILER.anchors
                .entry(#fn_name.to_string())
                .and_modify(|x| x.tsc_elapsed += read_cpu_timer() - x.start_tsc)
                .or_insert(ProfileAnchor { start_tsc: read_cpu_timer(), tsc_elapsed: 0, hit_count: 1 });
            }
        }
    };

    match function.sig.output {
        ReturnType::Default => {
            let new_body: Block = parse_quote! {
                {
                #init_block
                #start_time_block
                #(#stmts)*
                #end_time_block
                }
            };

            function.block = Box::new(new_body);
        }
        _ => {
            let ret_stmt = stmts.pop().unwrap();
            let new_body: Block = parse_quote! {
                {
                #init_block
                #start_time_block
                #(#stmts)*
                #end_time_block
                #ret_stmt
                }
            };
            function.block = Box::new(new_body);
        }
    }

    return (quote! {#function}).into();
}
