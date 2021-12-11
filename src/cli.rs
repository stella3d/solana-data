use clap::{self, Arg, App};

pub(crate) struct CliArguments {
    pub task: String,
}

pub(crate) fn get_cli_args() -> CliArguments {
    let app = App::new("Solana Data Processing Playground")
    .version("0.1.0")
    .author("by: stellz")
    .about("solana data toys: rpc scraping & analysis")
    .arg(Arg::with_name("task")
             .short("t")
             .long("task")
             .takes_value(true)
             .help("Which sub-command to run"));

    let matches = app.get_matches();

    let task = matches.value_of("task").unwrap_or("").clone().to_owned();
    println!("\ntask: {}\n", task);

    CliArguments { task }
}