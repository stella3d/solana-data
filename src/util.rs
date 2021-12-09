use std::time::{Duration, Instant};
use std::fmt::{Debug, Display};

pub struct TimedData<T> {
    pub time: Duration,
    pub data: T
}

// helper for timing code execution
pub fn timer<T: FnOnce() -> ()>(func: T) -> Duration {
    let start = Instant::now();
    func();
    return start.elapsed();
}

pub fn timer_log_ms<T: FnOnce() -> ()>(prefix: &str, func: T) -> Duration {
    let elapsed = timer(func);
    println!("{}  -  {}ms", prefix, elapsed.as_millis());
    elapsed
}

pub fn time_run<T: FnOnce() -> R, R>(func: T) -> TimedData<R> {
    let start = Instant::now();
    let returned = func();
    return TimedData { time: start.elapsed(), data: returned }
}

pub fn log_err<E: Debug + Display>(e: E) { eprintln!("{}", e); }

pub fn log_err_none<T, E: Debug + Display>(e: E) -> Option<T> { eprintln!("{}", e); None }