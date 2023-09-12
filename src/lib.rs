pub use macros::profile_zone;
use std::{arch::x86_64::_rdtsc, mem, collections::BTreeMap};
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

#[derive(Debug)]
pub struct Timer {
    pub os_freq: u64,
    pub os_timer: u64,
    pub cpu_timer: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            os_freq: os_freq(),
            os_timer: read_os_timer(),
            cpu_timer: read_cpu_timer(),
        }
    }
}

pub fn os_freq() -> u64 {
    unsafe {
        let mut freq = mem::zeroed();
        QueryPerformanceFrequency(&mut freq);
        return *freq.QuadPart() as u64;
    }
}

pub fn read_os_timer() -> u64 {
    unsafe {
        let mut os_timer = mem::zeroed();
        QueryPerformanceCounter(&mut os_timer);
        return *os_timer.QuadPart() as u64;
    }
}

pub fn read_cpu_timer() -> u64 {
    unsafe {
        return _rdtsc();
    }
}

/**
 * Give a close approximation based on the high precision timers above
 * Uses a 100ms delay to approximate the cpu frequency
 */
pub fn cpu_freq() -> u64 {
    let milis_to_wait = 100;
    let os_freq = os_freq();
    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    let mut os_elapsed = 0u64;
    let os_wait_time = os_freq * milis_to_wait / 1000;
    while os_elapsed < os_wait_time {
        os_elapsed = read_os_timer() - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let mut cpu_freq = 0u64;
    if os_elapsed > 0 {
        cpu_freq = os_freq * cpu_elapsed / os_elapsed;
    }

    return cpu_freq;
}

//start will be the current time of rdtsc when the macro is called
//count will be the number of times the profiled block was called
//elapsed will be the will be the sum of all times the block was profiled
#[derive(Debug)]
pub struct TimedBlock {
    pub start: usize,
    pub elapsed: usize,
    pub count: usize,
}

pub static mut TIMED_BLOCK: BTreeMap<String,TimedBlock> = BTreeMap::new();

#[macro_export]
macro_rules! start_block {
    ($block_name:literal) => {
        unsafe {
            use cpu_timer::*;
            let _start = cpu_timer::read_cpu_timer();
            let _name = $block_name;
            TIMED_BLOCK.entry(_name.to_string())
            .and_modify(|x| {
                let block_end = x.start;
                x.start = read_cpu_timer() as usize;
                x.elapsed += block_end - x.start;
                x.count += 1;
            })
            .or_insert(TimedBlock { start: read_cpu_timer() as usize, elapsed: 0, count: 1 });
        }
    };
}

#[macro_export]
macro_rules! end_block {
    ($block_name:literal) => {
        unsafe {
            use cpu_timer::*;
            let _end = cpu_timer::read_cpu_timer() as usize;
            let _name = $block_name;
            TIMED_BLOCK.entry(_name.to_string())
            .and_modify(|x| {                
                x.elapsed += _end - x.start;
            });
        }
    };
}

#[macro_export]
macro_rules! print_profile {
    () => {
        unsafe {            
            use cpu_timer::*;
            let _end_time = read_cpu_timer();
            TIMED_BLOCK.iter_mut().filter(|x| x.1.elapsed == 0).for_each(|x| {
                x.1.elapsed = _end_time as usize - x.1.start;
            });
            let profile_block = TIMED_BLOCK.get_mut("main").unwrap();
            let cpu_freq = cpu_freq();
            println!("Total main: {:.4}ms", (profile_block.elapsed as f64 / cpu_freq as f64) * 1000f64 );
            let mut acc_total = 0;
            for block in &TIMED_BLOCK {
                if block.0 == "main" {
                    continue;
                }
                acc_total += block.1.elapsed;
                println!("{} [{}] took: {}, {:.4}%", block.0, block.1.count, block.1.elapsed, (block.1.elapsed as f64 / profile_block.elapsed as f64) * 100f64);
            }
            let diff = profile_block.elapsed.abs_diff(acc_total);
            println!("profiled total: {} , diff: {}, {:.4}%", acc_total, diff, (acc_total as f64 / profile_block.elapsed as f64) * 100f64 );
            for b in &TIMED_BLOCK {
                println!("{:?}",b);
            }
        }
    };
}
