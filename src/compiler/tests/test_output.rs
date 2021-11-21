use insta::*;
use std::{fs, ops::Deref};

use compiler::{
    compiler_cli::{self, CompileResultSuccess},
    node_printer, parser,
    symbol_table::SymbolTable,
    tokenizer::{tokenize, tokens_to_xml},
};

#[test]
fn test_tokenizer_output() -> Result<(), Box<dyn std::error::Error>> {
    glob!("inputs/10/**/*.jack", |path| {
        let input = fs::read_to_string(path).unwrap();
        let is_gold = false;
        if is_gold {
            let gold_filename = path.to_owned().with_file_name(format!(
                "{}T.xml",
                path.file_stem().unwrap().to_string_lossy()
            ));
            let gold = fs::read_to_string(gold_filename).unwrap();
            assert_snapshot!(gold);
        } else {
            let tokens = tokenize(input.as_str());
            assert!(
                tokens.is_ok(),
                "Tokenizing error:\n{err}",
                err = tokens.unwrap_err()
            );
            let tokens_xml = tokens_to_xml(tokens.unwrap());
            assert_snapshot!(tokens_xml);
        }
    });
    Ok(())
}

#[test]
fn test_parser_output() -> Result<(), Box<dyn std::error::Error>> {
    glob!("inputs/10/**/*.jack", |path| {
        let is_gold = false;
        if is_gold {
            let gold_filename = path.to_owned().with_file_name(format!(
                "{}.xml",
                path.file_stem().unwrap().to_string_lossy()
            ));
            let gold = fs::read_to_string(gold_filename).unwrap();
            assert_snapshot!(gold);
        } else {
            let input = fs::read_to_string(path).unwrap();
            let result = parser::parse(input.as_str());
            assert!(
                result.is_ok(),
                "Parsing error:\n{err}",
                err = result.unwrap_err()
            );
            let result_xml = node_printer::result_to_xml(result.unwrap(), None);
            assert_snapshot!(result_xml);
        }
    });
    Ok(())
}

#[test]
fn test_parser_output_symbol_table() -> Result<(), Box<dyn std::error::Error>> {
    glob!("inputs/11/**/*.jack", |path| {
        let input = fs::read_to_string(path).unwrap();
        let result = parser::parse(input.as_str());
        assert!(
            result.is_ok(),
            "Parsing error:\n{err}",
            err = result.unwrap_err()
        );

        let mut sym_table = Some(SymbolTable::new());
        let result_xml = node_printer::result_to_xml(result.unwrap(), sym_table.as_mut());
        assert_snapshot!(result_xml);
    });
    Ok(())
}

#[test]
fn test_compiler_output() -> Result<(), Box<dyn std::error::Error>> {
    let base_path_ref = _macro_support::get_cargo_workspace(env!("CARGO_MANIFEST_DIR"));
    let base_path = base_path_ref.deref();

    glob!("inputs/**/*.jack", |path| {
        let result =
            compiler_cli::compile_file(path).map_err(|e| format!("Compiling error:\n{e}", e = e));
        let CompileResultSuccess { vm_code } = result.unwrap();
        assert_snapshot!(
            format!(
                "Compiler vm code: {path}",
                path = path.strip_prefix(base_path).unwrap().display()
            ),
            vm_code
        );
    });
    Ok(())
}

#[test]
fn test_compiler_examples() -> Result<(), Box<dyn std::error::Error>> {
    let path_workspace = _macro_support::get_cargo_workspace(env!("CARGO_MANIFEST_DIR"));
    let base_pathbuf = path_workspace.join("examples");
    let base_path = base_pathbuf.as_path();

    _macro_support::glob_exec(base_path, "**/*.jack", |path| {
        let result =
            compiler_cli::compile_file(path).map_err(|e| format!("Compiling error:\n{e}", e = e));
        let CompileResultSuccess { vm_code } = result.unwrap();
        assert_snapshot!(
            format!(
                "Compiler vm code example: {path}",
                path = path.strip_prefix(base_path).unwrap().display()
            ),
            vm_code
        );
    });
    Ok(())
}
