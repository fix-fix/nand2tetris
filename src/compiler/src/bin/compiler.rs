use std::{env, error::Error};

use ::compiler::{compiler_cli, config::Config};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).map_err(|err| format!("Problem parsing arguments: {}", err))?;

    compiler_cli::run(&config).map_err(|err| {
        // Print error manually because `main` func error reporter preseves escapes
        println!("Application error:\n{}", err);
        ""
    })?;
    Ok(())
}
