use std::{error::Error, fs};

use crate::{compiler, config::Config, input, parser};

#[derive(Debug)]
pub struct CompileResultSuccess {
    pub vm_code: String,
}

pub fn compile_file(file: &std::path::Path) -> Result<CompileResultSuccess, Box<dyn Error>> {
    let vm_code = compiler::compile_program(
        parser::parse(fs::read_to_string(file)?.as_str())
            .map_err(|e| format!("Error parsing file {:?}:\n{}", file, e))?,
    )?;
    Ok(CompileResultSuccess { vm_code })
}

pub fn run_for_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = input::get_files(config.source_path.clone());
    for file in files {
        // eprintln!("Compiling file: {:?}", file.as_path());
        let result = compile_file(&file)?;

        let mut target_path = file.clone();
        target_path.set_extension("vm");
        fs::write(target_path.as_path(), result.vm_code)?;
        // eprintln!("Wrote to: {}", target_path.as_path().display());
    }
    Ok(())
}
