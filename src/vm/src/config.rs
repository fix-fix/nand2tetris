pub struct Config {
    pub source_path: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Self, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let source_path = args[1].clone();
        Ok(Config { source_path })
    }
}
