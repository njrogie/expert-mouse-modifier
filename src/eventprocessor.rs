use std::{collections::HashMap, path::PathBuf, fs, io};
use evdev::InputEvent;
use log::{info,error};
use serde::Deserialize;
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct CmdMap {
    name_filter: String,
    base_code: i32,
    #[serde_as(as = "Vec<(_,_)>")]
    button_map: HashMap<u16,u16>
}

impl CmdMap {
    // create a new CmdMap from the required location, OR default map.
    pub fn new(loc: PathBuf) -> CmdMap {
        info!("Loading stored map from location.");
        match CmdMap::load_stored_map(loc) {
            Ok(map) => return map,
            Err(e) => {
                error!("{}", e);
                return CmdMap::default();
            }
        }
    }

    pub fn default() -> CmdMap {
        CmdMap {
            name_filter: String::from(""),
            base_code: 0,
            button_map: HashMap::new()
        }
    }

    pub fn get_name_filter(&self) -> String {
        self.name_filter.clone()
    }

    // translate one InputEvent to another. If no mapped event, throw err
    pub fn translate_command(&self, inp: InputEvent) -> Result<InputEvent, std::io::Error> {
        /* 
            Commands have 3 total input events apiece
            1.) KEY_3 [BASE_OFFSET + CODE]
            2.) [CODE] [1 for Press, 0 for Release]
            3.) KEY_RESERVED 0
         */

        fn button_not_contained<T: core::fmt::Display>(button: T) -> std::io::Error {
            std::io::Error::new(
                io::ErrorKind::NotFound,
                format!("Button not contained in map: {}", button)
            )
        }

        // case 1: if we're doing a reserved code, no change
        if inp.code() == evdev::Key::KEY_RESERVED.0 {
            return Ok(inp)
        } else if inp.code() == evdev::Key::KEY_3.0 {
            // case 2: if we are doing the KEY_3, subtract the offset from the code 
            // and map accordingly
            let value_temp = inp.value() - self.base_code;
            let value_temp = value_temp as u16;

            if !self.button_map.contains_key(&value_temp) {
                return Err(button_not_contained(value_temp));
            }
            return Ok(
                InputEvent::new(
                    inp.event_type(),
                    inp.code(), 
                    // at the end, reinsert the offset
                    self.button_map[&value_temp] as i32 + self.base_code
            ));

        } else {
            // case 3: otherwise follow logic for mapped key.
            let mapped_out = self.button_map.get(&inp.code());
            match mapped_out {
                Some(it) => {
                    Ok(InputEvent::new(
                        inp.event_type(),
                        *it, 
                        inp.value()
                    ))
                }, 
                None => {
                    // we didn't match
                    Err(button_not_contained(inp.code()))
                }
            }
        }
    }

    fn load_stored_map(location: PathBuf) -> Result<CmdMap, std::io::Error> {
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
        let map = CmdMap::new(get_test_file());
        assert!(map.button_map.len() > 0);
        assert!(map.button_map[&274] == 275);
    }
    #[test]
    fn translate_test_command() {
        let map = CmdMap::new(get_test_file());

        let test_cmd = InputEvent::new(evdev::EventType::KEY, 274, 1);

        assert!(map.translate_command(test_cmd).unwrap().code() == 275);
        assert!(map.translate_command(test_cmd).unwrap().value() == 1);
        assert!(map.translate_command(test_cmd).unwrap().event_type() == evdev::EventType::KEY);
    }

    fn get_test_file() -> PathBuf {
        String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test_cmds.json")).into()
    }
}