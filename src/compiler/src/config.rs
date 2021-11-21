pub struct Config {
    pub output_tokens: bool,
    pub source_path: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        if args.len() < 2 {
            return Err("not enough arguments".into());
        }

        let source_path = args[1].clone();
        Ok(Config {
            output_tokens: true,
            source_path,
        })
    }

    pub fn for_path(source_path: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            output_tokens: false,
            source_path,
        })
    }
}
