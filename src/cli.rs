use std::{io, path::PathBuf, process::exit};

use crate::DataStorage;

pub fn choose_file(data_storage: DataStorage) -> PathBuf {
    let files = data_storage.get_list_files();

    println!("Choose a file to pull configuration from:");
    for (idx, file) in files.iter().enumerate() {
        println!("{}. {}", idx + 1, file.display());
    }

    let mut choice = String::new();
    
    loop {
        io::stdin().read_line(&mut choice).unwrap();
        if choice.trim().starts_with("q"){
            exit(1);
        } else {
            match choice.trim().parse::<u16>().ok() {
                Some(ch) => {
                    let idx = usize::from(ch) - 1;
                    if idx < files.len() {
                        let chosen_path = files[idx].clone();  
                        return chosen_path     
                    } else {
                        println!("Not a choice that you can make, try again.");
                    }
                }, 
                None => {
                    println!("Not a number, please choose a number.")
                }
            }
        }
        choice = String::new();
    }
    // await the user input 
}