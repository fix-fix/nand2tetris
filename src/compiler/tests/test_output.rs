use insta::*;
use std::fs;

use compiler::{
    parser,
    tokenizer::{tokenize, tokens_to_xml},
};

// Workaround for https://github.com/mitsuhiko/insta/issues/119
macro_rules! glob_ {
    ($glob:expr, $closure:expr) => {{
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../")
            .join(file!())
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap();
        $crate::_macro_support::glob_exec(&base, $glob, $closure);
    }};
}

#[test]
fn test_tokenizer_output() -> Result<(), Box<dyn std::error::Error>> {
    glob_!("inputs/**/*.jack", |path| {
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
    glob_!("inputs/**/*.jack", |path| {
        let is_gold = true;
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
            let result_xml = parser::result_to_xml(result.unwrap());
            assert_snapshot!(result_xml);
        }
    });
    Ok(())
}
