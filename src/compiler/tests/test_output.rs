use insta::*;
use std::fs;

use compiler::{
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
