use std::time::{Duration, Instant};

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

pub fn time_run<T: FnOnce() -> R, R>(func: T) -> TimedData<R> {
    let start = Instant::now();
    let returned = func();
    return TimedData { time: start.elapsed(), data: returned }
}