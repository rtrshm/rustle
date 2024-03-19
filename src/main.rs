use log::LevelFilter;
use simple_logging;
use std::env;

pub mod control;
pub mod model;
pub mod view;

fn main() -> color_eyre::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    simple_logging::log_to_file("test.log", LevelFilter::Info)?;

    if let Err(e) = control::run() {
        println!("{}", e)
    }
    Ok(())
}
