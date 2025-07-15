use clap::{Arg, ArgAction, Command};
use std::process;

mod commands;
mod config;
mod djot;
mod fs;

#[tokio::main]
async fn main() {
    let matches = Command::new("arrow")
        .version("6.0")
        .about("a simple static site generator")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("serve")
                .about("start a local server and watch for changes")
                .alias("s")
                .arg(
                    Arg::new("port")
                        .long("port")
                        .value_name("PORT")
                        .help("specify the port to serve on")
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("entry")
                        .long("entry")
                        .short('e')
                        .value_name("ENTRY")
                        .help("specify the workspace key (e.g., site, notes)")
                        .required(true)
                        .action(ArgAction::Set),
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
                        .required(true)
                        .action(ArgAction::Set),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("list status of all source djot files")
                .alias("st")
                .arg(
                    Arg::new("entry")
                        .long("entry")
                        .short('e')
                        .value_name("ENTRY")
                        .help("specify the workspace key (e.g., site, notes)")
                        .required(true)
                        .action(ArgAction::Set),
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
                        .required(true)
                        .action(ArgAction::Set),
                ),
        )
        .get_matches();

    let result = match matches.subcommand() {
        Some(("serve", sub_m)) => {
            let port = sub_m
                .get_one::<String>("port")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);

            let entry = sub_m.get_one::<String>("entry").unwrap();
            commands::serve_command(port, entry).await
        }
        Some(("new", sub_m)) => {
            let entry = sub_m.get_one::<String>("entry").unwrap();
            commands::new_command(entry)
        }
        Some(("status", sub_m)) => {
            let entry = sub_m.get_one::<String>("entry").unwrap();
            commands::status_command(entry)
        }
        Some(("build", sub_m)) => {
            let entry = sub_m.get_one::<String>("entry").unwrap();
            commands::build_command(entry).await
        }
        _ => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
