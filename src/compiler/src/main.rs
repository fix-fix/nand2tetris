use std::env;

use compiler::{config, run};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config =
        config::Config::new(&args).map_err(|err| format!("Problem parsing arguments: {}", err))?;

    run(&config).map_err(|err| {
        format!("Application error: {}", err).into()
    })
}
