use std::{env, error::Error, fs};

use compiler::{config::Config, input, node_printer, parser, symbol_table::SymbolTable};

fn run(config: &Config, should_resolve_symbols: bool) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    for file in files {
        eprintln!("Parsing file: {:?}", file.as_path());
        let mut symbol_table = if should_resolve_symbols {
            Some(SymbolTable::new())
        } else {
            None
        };
        let tokens_result = node_printer::result_to_xml(
            parser::parse(fs::read_to_string(file.as_path())?.as_str())
                .map_err(|e| format!("Error parsing file {:?}:\n{}", file.as_path(), e))?,
            symbol_table.as_mut(),
        );
        let mut target_path = file.clone();
        target_path.set_extension("parser-out.xml");
        fs::write(target_path.as_path(), tokens_result)?;
        println!("Wrote to: {}", target_path.as_path().display());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).map_err(|err| format!("Problem parsing arguments: {}", err))?;
    let should_resolve_symbols = args.contains(&"--resolve-symbols".into());

    run(&config, should_resolve_symbols).map_err(|err| {
        // Print error manually because `main` func error reporter preseves escapes
        println!("Application error:\n{}", err);
        "Error"
    })?;
    Ok(())
}
