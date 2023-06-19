use std::{arch::x86_64::_rdtsc, mem};
use winapi::um::profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency};

pub struct Timer {
    pub os_freq: u64,
    pub os_timer: u64,
    pub cpu_timer: u64,
    pub cpu_freq: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            os_freq: os_freq(),
            os_timer: read_os_timer(),
            cpu_timer: read_cpu_timer(),
            cpu_freq: cpu_freq(),
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

//Give a close approximation based on the high precision timers above
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
