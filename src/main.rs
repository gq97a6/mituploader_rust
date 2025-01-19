use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;
use std::fs;
//use std::path::PathBuf;
use pause_console::pause_console;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Get the current working directory
    let dir = env::current_dir()?;
    //let dir = PathBuf::from(r"C:\Users\Me\Desktop\mituploader_rust\src\files"); //dev tmp

    //Get list of users
    let users_file = dir.join("users.txt");
    if !users_file.exists() {
        eprintln!("No users.txt found in current directory. Exiting.");
        pause_console!("Press Enter to close...");
        return Ok(());
    }

    let users_content = fs::read_to_string(users_file)?;
    let mut users: HashMap<String, Vec<String>> = HashMap::new();

    for line in users_content.lines() {
        let parts: Vec<_> = line.trim().split(':').collect();
        if parts.len() == 2 {
            let blocks: Vec<_> = parts[1].split('-').map(String::from).collect();
            users.insert(parts[0].to_string(), blocks);
        }
    }

    if users.is_empty() {
        eprintln!("No users found. Exiting.");
        pause_console!("Press Enter to close...");
        return Ok(());
    }

    //Get list of files
    let mut file_paths = vec![];
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();

        match path.extension() {
            Some(extension) if extension == "aia" => {
                file_paths.push(path);
            }
            _ => {}
        }
    }

    if file_paths.is_empty() {
        eprintln!("No files to upload found. Exiting.");
        pause_console!("Press Enter to close...");
        return Ok(());
    }

    let client = Client::builder()
        .cookie_store(true)
        .build()?;

    for (user, blocks) in &users {
        if blocks.len() < 4 {
            eprintln!("Invalid data for user: {}", user);
            continue;
        }

        for file_path in &file_paths {
            let params = [
                ("A", &blocks[0]),
                ("B", &blocks[1]),
                ("C", &blocks[2]),
                ("D", &blocks[3]),
                ("locale", &"en".to_string()),
                ("revisit", &"true".to_string()),
                ("host", &"code.appinventor.mit.edu".to_string()),
            ];

            //Login
            let login_response = client.post("https://code.appinventor.mit.edu/login")
                .form(&params)
                .send()?;

            if !login_response.status().is_success() {
                eprintln!("Failed to login as {}", user);
                break;
            }

            let file_name = file_path.file_name()
                .ok_or("Failed to get file name")?
                .to_string_lossy();
            
            let project_name = file_name.trim_end_matches(".aia");
            let upload_url = format!("https://code.appinventor.mit.edu/ode/upload/project/{}", project_name);

            // Upload file
            let upload_response = client.post(&upload_url)
                .multipart(reqwest::blocking::multipart::Form::new()
                .file("uploadProjectArchive", file_path)?)
                .send()?;

            if !upload_response.status().is_success() {
                eprintln!("Failed to upload {} to {}", file_name, user);
                break;
            }

            println!("Sent {} to {}", file_name, user);
        }
    }

    pause_console!("Press Enter to close...");
    Ok(())
}
