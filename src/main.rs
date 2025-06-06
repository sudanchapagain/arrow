use clap::{Arg, Command};
use std::process;

mod cmd;

fn main() {
    let matches = Command::new("arrow")
        .version("6.0")
        .about("a simple static site generator")
        .subcommand(
            Command::new("serve")
                .about("start a local server and watch for changes")
                .alias("s")
                .arg(
                    Arg::new("port")
                        .long("port")
                        .value_name("PORT")
                        .help("specify the port to serve on")
                        .num_args(1),
                )
                .arg(
                    Arg::new("entry")
                        .long("entry")
                        .short('e')
                        .value_name("ENTRY")
                        .help("specify the workspace key (e.g., site, notes)")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("new")
                .about("create a new entry in workspace")
                .alias("n")
                .arg(
                    Arg::new("entry")
                        .long("entry")
                        .short('e')
                        .value_name("ENTRY")
                        .help("specify the workspace key (e.g., site, notes)")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("list status of all source markdown files")
                .alias("st")
                .arg(
                    Arg::new("entry")
                        .long("entry")
                        .short('e')
                        .value_name("ENTRY")
                        .help("specify the workspace key (e.g., site, notes)")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("build the static source files")
                .alias("b")
                .arg(
                    Arg::new("entry")
                        .long("entry")
                        .short('e')
                        .value_name("ENTRY")
                        .help("specify the workspace key (e.g., site, notes)")
                        .num_args(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("serve", sub_m)) => {
            let port = sub_m
                .get_one::<String>("port")
                .and_then(|p| p.parse::<i32>().ok())
                .unwrap_or(0);

            let entry = sub_m
                .get_one::<String>("entry")
                .map(String::as_str)
                .unwrap_or("");

            cmd::serve(port, entry);
        }
        Some(("new", sub_m)) => {
            let entry = sub_m
                .get_one::<String>("entry")
                .map(String::as_str)
                .unwrap_or("");
            cmd::new_entry(entry);
        }
        Some(("status", sub_m)) => {
            let entry = sub_m
                .get_one::<String>("entry")
                .map(String::as_str)
                .unwrap_or("");
            cmd::status(entry);
        }
        Some(("build", sub_m)) => {
            let entry = sub_m
                .get_one::<String>("entry")
                .map(String::as_str)
                .unwrap_or("");
            cmd::build(entry);
        }
        _ => {
            eprintln!("use --help to see available ocmmands");
            process::exit(1);
        }
    }
}
