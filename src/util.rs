use std::{time::{Duration, Instant}, fmt::{Debug, Display}};


pub(crate) const MEGABYTE: u64 = 1024 * 1024;

pub fn loop_task<F: Fn() -> ()>(total_time: Duration, loop_fn: F) {
    let start = Instant::now();
    let end = start + total_time;
    while Instant::now() < end {
        loop_fn();
    }
    println!("loop task finished after {:3} seconds", start.elapsed().as_secs_f32());
}


// EXECUTION TIMING
pub struct TimedData<T> {
    pub time: Duration,
    pub data: T
}

pub fn timer<T: FnOnce() -> ()>(func: T) -> Duration {
    let start = Instant::now();
    func();
    return start.elapsed();
}

pub fn time_run<T: FnOnce() -> R, R>(func: T) -> TimedData<R> {
    let start = Instant::now();
    let returned = func();
    TimedData { time: start.elapsed(), data: returned }
}

pub(crate) fn minutes_duration(minutes: u64) -> Duration {
    Duration::from_secs(60 * minutes)
}


// LOGGING (errors)
pub fn log_err<E: Debug + Display>(e: &E) { eprintln!("{}", e); }

pub fn log_err_none<T, E: Debug + Display>(e: E) -> Option<T> { eprintln!("{}", e); None }

// LOGGING (slices)
pub(crate) fn println_each_indent<T: Display>(data: &[T], end_newline: bool) {
    data.iter().for_each(|d| println!("    {}", d));
    if end_newline { println!("") }
}

pub(crate) fn dbg_println_each_indent<T: Debug>(data: &[T], end_newline: bool) {
    data.iter().for_each(|d| println!("    {:?}", d));
    if end_newline { println!("") }
}