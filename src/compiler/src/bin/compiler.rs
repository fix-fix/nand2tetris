use std::{env, error::Error, fs};

use ::compiler::{compiler, config::Config, input, parser};

fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    for file in files {
        // eprintln!("Compiling file: {:?}", file.as_path());
        let vm_code = compiler::compile_program(
            parser::parse(fs::read_to_string(file.as_path())?.as_str())
                .map_err(|e| format!("Error parsing file {:?}:\n{}", file.as_path(), e))?
        )?;
        let mut target_path = file.clone();
        target_path.set_extension("vm");
        fs::write(target_path.as_path(), vm_code)?;
        // eprintln!("Wrote to: {}", target_path.as_path().display());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).map_err(|err| format!("Problem parsing arguments: {}", err))?;

    run(&config).map_err(|err| {
        // Print error manually because `main` func error reporter preseves escapes
        println!("Application error:\n{}", err);
        ""
    })?;
    Ok(())
}
