use evdev::{AttributeSet, RelativeAxisType, Device};
use std::fmt::Display;

pub struct DeviceInfo {
    pub axes: AttributeSet<RelativeAxisType>,
    pub mouse_buttons: AttributeSet<evdev::Key>
}

impl DeviceInfo  {
    pub fn new(device: &Device) -> DeviceInfo {
        let axes = device
            .supported_relative_axes()
            .unwrap();

        let mouse_buttons = device
            .supported_keys()
            .unwrap();
        
        // create new copies of device capabilities, since we prob want to edit them
        let mut new_axes_set = evdev::AttributeSet::<RelativeAxisType>::new();
        let it = axes.iter();
        for key in it {
            new_axes_set.insert(key);
        }

        let mut new_button_set = evdev::AttributeSet::<evdev::Key>::new();
        let it = mouse_buttons.iter();
        for key in it {
            new_button_set.insert(key);
        }

        // add other keys we want to possibly function
        // NOTE- if you don't add a button here, even if you trigger the code
        // it won't function in the virtual device.
        new_button_set.insert(evdev::Key::BTN_EXTRA); // mouse 5

        DeviceInfo{
            axes: new_axes_set,
            mouse_buttons: new_button_set
        }
    }
}

impl Display for DeviceInfo{
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