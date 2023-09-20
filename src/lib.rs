pub use macros::profile_zone;
use std::{arch::x86_64::_rdtsc, collections::BTreeMap, mem};
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

#[derive(Debug)]
pub struct Timer {
    pub os_freq: usize,
    pub os_timer: usize,
    pub cpu_timer: usize,
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

pub fn os_freq() -> usize {
    unsafe {
        let mut freq = mem::zeroed();
        QueryPerformanceFrequency(&mut freq);
        return *freq.QuadPart() as usize;
    }
}

pub fn read_os_timer() -> usize {
    unsafe {
        let mut os_timer = mem::zeroed();
        QueryPerformanceCounter(&mut os_timer);
        return *os_timer.QuadPart() as usize;
    }
}

pub fn read_cpu_timer() -> usize {
    unsafe {
        return _rdtsc() as usize;
    }
}

/**
 * Give a close approximation based on the high precision timers above
 * Uses a 100ms delay to approximate the cpu frequency
 */
pub fn cpu_freq() -> usize {
    let milis_to_wait = 100;
    let os_freq = os_freq();
    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    let mut os_elapsed = 0;
    let os_wait_time = os_freq * milis_to_wait / 1000;
    while os_elapsed < os_wait_time {
        os_elapsed = read_os_timer() - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let mut cpu_freq = 0;
    if os_elapsed > 0 {
        cpu_freq = os_freq * cpu_elapsed / os_elapsed;
    }

    return cpu_freq;
}

#[derive(Debug)]
pub struct ProfileAnchor {
    pub start_tsc: usize,
    pub tsc_elapsed: usize,
    pub hit_count: usize,
}

#[derive(Debug)]
pub struct Profiler {
    pub start_tsc: usize,
    pub end_tsc: usize,
    pub anchors: BTreeMap<String, ProfileAnchor>,
}

pub static mut PROFILER: Profiler = Profiler {
    start_tsc: 0,
    end_tsc: 0,
    anchors: BTreeMap::new(),
};

#[macro_export]
macro_rules! start_block {
    ($block_name:literal) => {
        unsafe {
            use cpu_timer::{read_cpu_timer, ProfileAnchor, PROFILER};
            let _start = read_cpu_timer();
            let _name = $block_name;
            PROFILER
                .anchors
                .entry(_name.to_string())
                .and_modify(|x| {
                    x.tsc_elapsed += read_cpu_timer() - x.start_tsc;
                    x.hit_count += 1;
                })
                .or_insert(ProfileAnchor {
                    start_tsc: read_cpu_timer(),
                    tsc_elapsed: 0,
                    hit_count: 1,
                });
        }
    };
}

#[macro_export]
macro_rules! end_block {
    ($block_name:literal) => {
        unsafe {
            use cpu_timer::{read_cpu_timer, ProfileAnchor, PROFILER};
            let _name = $block_name;
            PROFILER.anchors.entry(_name.to_string()).and_modify(|x| {
                x.tsc_elapsed += read_cpu_timer() - x.start_tsc;
            });
        }
    };
}

#[macro_export]
macro_rules! print_profile {
    () => {
        unsafe {
            use cpu_timer::{cpu_freq, read_cpu_timer, ProfileAnchor, PROFILER};
            PROFILER.end_tsc = read_cpu_timer();
            let cpu_freq = cpu_freq();

            let total_cpu_elapsed = PROFILER.end_tsc - PROFILER.start_tsc;

            println!(
                "Total Time: {:.4}ms (CPU freq {})",
                1000f64 * total_cpu_elapsed as f64 / cpu_freq as f64,
                cpu_freq
            );

            for (name, anchor) in &PROFILER.anchors {
                println!(
                    "{} [{}] took: {}, {:.2}%",
                    name,
                    anchor.hit_count,
                    anchor.tsc_elapsed,
                    (anchor.tsc_elapsed as f64 / total_cpu_elapsed as f64) * 100f64
                );
            }
        }
    };
}
