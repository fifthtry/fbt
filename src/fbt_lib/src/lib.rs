use std::path::PathBuf;

pub mod types;

pub fn test_all() -> anyhow::Result<bool> {
    for dir in std::fs::read_dir("./tests")? {
        test_one(dir?)?;
    }

    Ok(true)
}

fn get_file_path(
    curr_dir: &std::fs::DirEntry,
    file_obj: &str,
    is_dir: bool,
) -> anyhow::Result<Option<PathBuf>> {
    for dir in std::fs::read_dir(curr_dir.path())? {
        let entry = dir?;
        if is_dir {
            if entry.file_type()?.is_dir()
                && entry.file_name().to_str().unwrap_or("").contains(file_obj)
            {
                return Ok(Some(entry.path()));
            }
        } else {
            if entry.file_name().to_str().unwrap_or("").contains(file_obj) {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}

fn test_one(entry: std::fs::DirEntry) -> anyhow::Result<bool> {
    let file_name = entry.file_name();
    let test_dir = match file_name.to_str() {
        Some(f) => f,
        None => {
            eprintln!("cant convert directory to str: {:?}", entry);
            return Ok(false);
        }
    };

    if !test_dir.starts_with('.') && entry.file_type()?.is_dir() {
        // Not testing fbt as of now
        if test_dir.contains("fbt") {
            return Ok(false);
        }

        println!("current folder {:?}", entry.path());
        let input_path = match get_file_path(&entry, "input", true) {
            Ok(res) => {
                if let Some(path) = res {
                    path
                } else {
                    eprintln!("not a valid test case");
                    return Ok(false);
                }
            }
            _ => {
                eprintln!("not a valid test case");
                return Ok(false);
            }
        };
        let cmd_toml_path = match get_file_path(&entry, "cmd.toml", false) {
            Ok(res) => {
                if let Some(path) = res {
                    path
                } else {
                    eprintln!("not a valid test case");
                    return Ok(false);
                }
            }
            _ => {
                eprintln!("not a valid test case");
                return Ok(false);
            }
        };

        println!("input: {:?}, cmd.toml {:?}", input_path, cmd_toml_path);

        let contents = std::fs::read_to_string(cmd_toml_path)?;
        let test_cmd: crate::types::TestCommand = toml::from_str(&contents)?;
        println!("Command: {:?}", test_cmd);

        let args: Vec<&str> = test_cmd.cmd.split(' ').collect();
        let mut cmd = std::process::Command::new(args[0]);
        cmd.current_dir(input_path);
        //will need to add code to handle multiple args
        cmd.arg(args[1]);
        let result = cmd.output()?;
        println!("cmd result {:?}", result);
        if String::from_utf8(result.stdout)? == test_cmd.stdout.trim() && result.status.success() {
            println!("Passed");
        } else {
            println!("Failed {:?}", String::from_utf8(result.stderr)?.trim());
        }
    }

    Ok(true)
}