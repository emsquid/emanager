use clap::Parser;

mod acpi;
mod args;
mod battery;
mod brightness;
mod hypr;
mod manager;
mod notifier;
mod logger;
mod system;
mod utils;
mod volume;
mod wifi;

fn main() {
    let args = args::Args::parse();

    let result = match args.command {
        args::Command::Daemon => manager::Manager::daemon(),
        _ => manager::Manager::handle(args.command),
    };

    if let Err(e) = result {
        eprintln!("{e}");
    }
}
