use std::path;

pub fn get_files(path: String) -> Vec<path::PathBuf> {
    let arg_path = path::PathBuf::from(path);
    let files = if arg_path.is_dir() {
        arg_path
            .read_dir()
            .expect("Read dir failed")
            .filter_map(Result::ok)
            .filter_map(|f: std::fs::DirEntry| {
                if f.file_name().into_string().ok()?.ends_with(".jack") {
                    Some(f.path())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![arg_path]
    };
    files
}
