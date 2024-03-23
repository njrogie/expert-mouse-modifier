use log::info;
use std::{fmt::Display, fs};
use evdev::{uinput::VirtualDeviceBuilder, AttributeSetRef, Device, RelativeAxisType};

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // find the devices linked to kensington
    let device = find_devices()?[0].clone();
    let mut device = Device::open(device)?;
    
    info!("Grabbing device {}", device.name().unwrap());
    let info = DeviceInfo::new(&device);    
    info!("{}", info);
    
    // grab the device and begin event transcription
    match device.grab() {
        Ok(_) => {
            manage_events(device);
            return Ok(())
        },
        Err(e) => {
            return Err(Box::new(e));
        }
    }
}

struct DeviceInfo<'a> {
    axes: &'a AttributeSetRef<RelativeAxisType>,
    mouse_buttons: &'a AttributeSetRef<evdev::Key>
}

impl<'a> DeviceInfo<'a> {
    fn new(device: &Device) -> DeviceInfo {
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


fn find_devices() -> Result<Vec<String>,Box<dyn std::error::Error>> {
    // find the devices linked to kensington
    let devices = fs::read_dir("/dev/input/by-id")?;
    let mut kensington_devices: Vec<String> = vec![];
    for dev in devices{
        let file_name = dev?.path().into_os_string().into_string().unwrap();
        let is_kensington = file_name.to_ascii_lowercase().contains("kensington");
        let contains_event = file_name.to_ascii_lowercase().contains("event-mouse"); // we want regular event, not mouse

        if is_kensington && contains_event {
            // add that device to the list of devices found.
            kensington_devices.push(String::from(file_name));
        } 
    }

    info!("Found {} Kensington Devices.", kensington_devices.len());
    for device in &kensington_devices {
        info!("\t{}", device);
    }
    Ok(kensington_devices)
}

fn manage_events(mut device: Device) -> Result<(),std::io::Error>{
    info!("Creating virtual device...");
    let info = DeviceInfo::new(&device);
    let virt_dev = VirtualDeviceBuilder::new()?;
    let mut virt_dev = virt_dev.name("Remapped Trackball Mouse")
        .with_keys(info.mouse_buttons)?
        .with_relative_axes(info.axes)?
        .build()?;

    info!("Begin event read...");
    loop {
        let events = device.fetch_events()?;
        info!("------------");
        let mut events_slice = vec![];
        for event in events {
            info!("{:?}, {}", event.code(), event.value());
            events_slice.push(event);
        }
        virt_dev.emit(&events_slice)?;
    }
}
