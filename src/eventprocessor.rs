use std::{collections::HashMap, fs};
use log::{info,error};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CmdMap {
    base_code: i32,
    button_map: Vec<(i32,i32)>
}

impl CmdMap {
    // create a new CmdMap from the required location, OR default map.
    pub fn new(loc: String) -> CmdMap {
        info!("Loading stored map from location.");
        match CmdMap::load_stored_map(loc) {
            Ok(map) => return map,
            Err(e) => {
                error!("{}", e);
                return CmdMap::default();
            }
        }
    }

    pub fn default() -> CmdMap{
        CmdMap {
            base_code: 0,
            button_map: vec![]
        }
    }

    fn load_stored_map(location: String) -> Result<CmdMap, std::io::Error> {
        /*
            it turns out that the only thing we need to record in the file
            is the key code. other parts of the click that are sent are simply
            offset by the keycode (at least, on linux)
         */

        // load the .json file from location
        let data = fs::read_to_string(location)?;
        let data: CmdMap = serde_json::from_str(data.as_str())?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn load_test_file() {
        let test_file = concat!(env!("CARGO_MANIFEST_DIR"), "/test_cmds.json");
        let path = String::from(test_file);
        let map = CmdMap::new(path);
        assert!(map.button_map.len() > 0);
        assert!(map.button_map[0].0 == 274);
        assert!(map.button_map[0].1 == 275);
    }
}