use clap::Parser;

mod acpi;
mod args;
mod brightness;
mod hypr;
mod manager;
mod notifier;
mod stater;
mod systemd;
mod utils;
mod volume;

fn main() {
    let args = args::Args::parse();

    let result = match args.command {
        args::Command::Start => manager::Manager::new().start(),
        _ => manager::Manager::send(args.command),
    };

    if let Err(e) = result {
        eprintln!("{e}");
    }
}
