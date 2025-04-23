use log::LevelFilter;
use simple_logging;
use std::env;

// pub mod model;
// pub mod view;

use rustle::run;

fn main() -> color_eyre::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    simple_logging::log_to_file("test.log", LevelFilter::Info)?;

    if let Err(e) = run() {
        println!("{}", e)
    }
    Ok(())
}
