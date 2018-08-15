use std::env;

pub fn resolve_settings_path() -> String {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => args[1].to_owned(),
        _ => {
            let mut dir = env::current_exe().expect("Cannot get current directory");
            dir.set_file_name("config.json");
            dir.to_str().expect("Invalid path").to_owned()
        }
    }
}