use cpu_timer::{end_block, print_profile, read_cpu_timer, start_block};
use macros::profile_zone;
use std::env;

#[profile_zone(init = true)]
fn main() {
    let mut milis_to_wait = 1000;
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        milis_to_wait = args[1].parse().unwrap();
    }

    let os_freq = cpu_timer::os_freq();
    println!("OS Freq: {os_freq}");

    let cpu_start = read_cpu_timer();
    let os_start = cpu_timer::read_os_timer();
    let mut os_end = 0;
    let mut os_elapsed = 0;
    let os_wait_time = os_freq * milis_to_wait / 1000;

    start_block!("loop");
    while os_elapsed < os_wait_time {
        os_end = cpu_timer::read_os_timer();
        os_elapsed = os_end - os_start;
    }
    end_block!("loop");

    start_block!("everything_else");
    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let cpu_freq = cpu_timer::cpu_freq();

    println!(
        "OS Timer: {} -> {} = {} elapsed",
        os_start, os_end, os_elapsed
    );
    println!("OS Seconds: {:0.4}", os_elapsed as f64 / os_freq as f64);

    println!(
        "CPU Timer: {} -> {} = {} elapsed",
        cpu_start, cpu_end, cpu_elapsed
    );
    println!("CPU Freq: {} (guessed)", cpu_freq);
    end_block!("everything_else");

    end_block!("main");
    print_profile!();
}
