use std::path::PathBuf;
use std::io;
use log::info;
pub struct DataStorage {
    data_files: Vec<PathBuf>
}

impl DataStorage {
    pub fn new(path: Option<PathBuf>) -> DataStorage {
        let mut read_data_files: Vec<PathBuf> = vec![];
        match get_data_files(path) {
            Ok(it) => {
                read_data_files = it;
            },
            Err(_) => {}
        }

        DataStorage{ data_files: read_data_files }
    }

    pub fn get_file(&self, idx: usize) -> Result<PathBuf, std::io::Error> {
        if !self.has_files() || idx > self.get_num_files() - 1 {
            dbg!("{:?}", &self.data_files);
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("{} is outside of the number of files", idx)));
        } else {
            return Ok(self.data_files[idx].clone());
        }
    }

    pub fn get_num_files(&self) -> usize {
        self.data_files.len()
    }

    pub fn has_files(&self) -> bool {
        self.get_num_files() != 0
    }
    
}

fn get_data_files(path: Option<PathBuf>) -> Result<Vec<PathBuf>, std::io::Error> {
    let data_dir = get_data_dir(path)?;
    info!("Data Dir: {}", data_dir.display());
    // create data_dir if not exists
    if !data_dir.exists() {

        //create the data_dir
        std::fs::create_dir(&data_dir)?;
    }

    // read through the files to get the config files
    let mut config_file_list: Vec<PathBuf> = vec![];
    let files = std::fs::read_dir(data_dir)?;
    for file in files {
        // add the absolute path of the file to the list
        config_file_list.push(file.unwrap().path());
    }

    Ok(config_file_list)
}

fn get_data_dir(prefix: Option<PathBuf>) -> Result<PathBuf, std::io::Error> {
    match prefix {
        Some(dir) => {
            let mut os_dir = dir.into_os_string();
            os_dir.push(".mousemod");
            
            return Ok(os_dir.into());
        }, 
        None => {
            return std::env::current_dir();
        }
    }
}