mod code;
pub mod config;
pub mod instruction;
mod parser;

use std::error::Error;
use std::fs;
use std::path;

fn process_files(config: &config::Config) -> Result<(path::PathBuf, String), Box<dyn Error>> {
    let source_path = path::PathBuf::from(&config.source_path);
    let file_metadata = source_path.metadata()?;
    let result = match file_metadata {
        m if m.is_file() => {
            let contents = fs::read_to_string(&config.source_path)?;
            let filename = source_path.clone();
            let filename_stem = filename
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid filename")?;
            let generated = code::generate_code(parser::create(&contents, filename_stem).parse());
            let mut target = source_path;
            target.set_extension("asm");
            let bootstrap = code::generate_bootstrap(filename_stem);
            Ok((target, format!("{}\n{}", bootstrap, generated)))
        }
        m if m.is_dir() => {
            let targets = fs::read_dir(&source_path)?;
            let generated = targets
                .filter_map(Result::ok)
                .filter(|f| f.file_name().into_string().unwrap().ends_with(".vm"))
                .map(|f| -> Result<_, Box<dyn Error>> {
                    let module_source = fs::read_to_string(f.path())?;
                    let filename = f.path();
                    let filename_stem = filename
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or("Invalid filename")?;
                    let generated =
                        code::generate_code(parser::create(&module_source, filename_stem).parse());
                    Ok(generated)
                })
                .filter_map(Result::ok)
                .collect::<Vec<_>>()
                .join("\n");
            let mut target = source_path.clone();
            let filename = source_path
                .file_stem()
                .and_then(|f| f.to_str())
                .ok_or("Invalid filename")?;
            target.push(path::Path::new(filename));
            target.set_extension("asm");
            let bootstrap = code::generate_bootstrap(filename);
            Ok((target, format!("{}\n{}", bootstrap, generated)))
        }
        _ => Err(format!("Invalid source: {}", source_path.to_string_lossy())),
    }?;
    Ok(result)
}

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let (target_path, generated) = process_files(config)?;
    fs::write(target_path, generated)?;
    Ok(())
}
