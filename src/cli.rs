use clap::{self, Arg, App};

pub(crate) struct CliArguments {
    pub task: String,
}

pub(crate) fn get_cli_args() -> CliArguments {
    let app = App::new("Solana Data Processing Playground")
    .version("0.1.0")
    .author("stellz")
    .about("personal solana data tools: rpc scraping & block analyzing")
    .arg(Arg::with_name("task")
             .short("t")
             .long("task")
             .takes_value(true)
             .help("Which sub-command to run"));
             /* 
    .arg(Arg::with_name("num")
             .short("n")
             .long("number")
             .takes_value(true)
             .help("Five less than your favorite number"));
             */
    let matches = app.get_matches();

    let task = matches.value_of("task").unwrap_or("").clone().to_owned();
    println!("\nThe task passed is: {}\n", task);

    CliArguments { task }
}