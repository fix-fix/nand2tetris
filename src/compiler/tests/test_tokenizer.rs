use insta::*;
use std::fs;

use compiler::tokenizer::{tokenize, tokens_to_xml};

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
fn test_tokenizer_output() {
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
            assert_snapshot!(tokens_to_xml(tokenize(input.as_str()).unwrap()));
        }
    });
}
