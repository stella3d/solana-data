use std::{time::{Duration, Instant}, fmt::{Debug, Display}, process::exit};


pub(crate) const MEGABYTE: usize = 1024 * 1024;

pub fn loop_task<F: Fn() -> ()>(total_time: Duration, loop_fn: F) {
    let start = Instant::now();
    let end = start + total_time;
    while Instant::now() < end {
        loop_fn();
    }
    println!("loop task finished after {:3} seconds", start.elapsed().as_secs_f32());
}

// run a task that we can't proceed without the success of, exit if it fails
pub(crate) fn do_or_die<F: FnOnce() -> Result<T, E>, T, E: Debug + Display>
    (task: F, err_msg: &str) -> T 
{
    match task() {
        Ok(output) => output,
        Err(e) => {
            log_err(&e);
            log_err(err_msg);
            exit(1);
        }}
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
pub fn log_err<E: Debug + Display + ?Sized>(e: &E) { eprintln!("{}", e); }

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