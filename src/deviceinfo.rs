use evdev::{AttributeSetRef, RelativeAxisType, Device};
use std::fmt::Display;

pub struct DeviceInfo<'a> {
    pub axes: &'a AttributeSetRef<RelativeAxisType>,
    pub mouse_buttons: &'a AttributeSetRef<evdev::Key>
}

impl<'a> DeviceInfo<'a> {
    pub fn new(device: &Device) -> DeviceInfo {
        let axes = device
            .supported_relative_axes()
            .unwrap();

        let mouse_buttons = device
            .supported_keys()
            .unwrap();
        
        DeviceInfo{
            axes: axes,
            mouse_buttons: mouse_buttons
        }
    }
}

impl Display for DeviceInfo<'_>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::from("\nKENSINGTON DEVICE INFO\n");
        out += "---------\n";
        out += "MOUSE AXES\n";
        let iter = self.axes.iter();
        for axis in iter {
            out += "\t";
            out += format!("{:?}",axis).as_str();
            out += "\n";
        }

        out += "---------\n";
        out += "MOUSE BUTTONS\n";
        let iter = self.mouse_buttons.iter();
        for button in iter {
            out += "\t";
            out += format!("{:?}",button).as_str();
            out += "\n";
        }
        write!(f, "{}", out)
    }
}