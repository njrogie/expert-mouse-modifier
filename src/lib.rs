use log::info;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::fs;
use evdev::{uinput::{VirtualDevice, VirtualDeviceBuilder}, Device, InputEvent};

mod deviceinfo;
mod eventprocessor;
mod localdata;

use deviceinfo::DeviceInfo;
use eventprocessor::CmdMap;
use localdata::DataStorage;

// Create and run the middleman routine for a mouse device
pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    // find and load the command map registry 
    let home_dir = Some(std::env::var("HOME_DIR")?.into());
    let data_storage = DataStorage::new(home_dir);

    // find the devices linked to the loaded map
    let cmd_map = CmdMap::new({
        if data_storage.has_files() {
            data_storage.get_file(0)?
        } else {
            std::env::current_dir()?
        }
    });

    let device = find_devices(&cmd_map)?[0].clone();
    let mut device = Device::open(device)?;   
    
    info!("Grabbing device {}", device.name().unwrap());
    let info = DeviceInfo::new(&device);    
    info!("{}", info);
    
    // grab the device and begin event transcription
    match device.grab() {
        Ok(_) => {
            manage_events(cmd_map, device).await;
            return Ok(())
        },
        Err(e) => {
            return Err(Box::new(e));
        }
    }
}

// Filter devices for the ones we want ("name filter" + "event-mouse" in input devices)
fn find_devices(cmd_map: &CmdMap) -> Result<Vec<String>,Box<dyn std::error::Error>> {
    let devices = fs::read_dir("/dev/input/by-id")?;
    let mut kensington_devices: Vec<String> = vec![];

    // Select the desired mouse device
    for dev in devices {
        let file_name = dev?.path().into_os_string().into_string().unwrap();
        let device_type = file_name.to_ascii_lowercase().contains(cmd_map.get_name_filter().as_str());
        let contains_event = file_name.to_ascii_lowercase().contains("event-mouse"); // we want regular event, not mouse
        if device_type && contains_event {
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

// Create the virtual device and begin processing events.
async fn manage_events(map: CmdMap, device: Device) {
    let (tx, rx) = mpsc::channel(100);
    let virt_dev = build_virtual_device(&device).unwrap();

    // spawn a worker thread that receives 'processed' events
    tokio::spawn(async move {
        write_virt_device(map, virt_dev, rx).await;
    });

    // run the read on the main thread
    read_device(device, tx).await;
}

// Build the virtual device that's going to receive processed events.
fn build_virtual_device(phys_device: &Device) -> Result<VirtualDevice, std::io::Error> {
    let mut info = DeviceInfo::new(&phys_device);
    // add mouse buttons as necessary
    info.mouse_buttons.insert(evdev::Key::BTN_EXTRA);
    let virt_dev = VirtualDeviceBuilder::new()?;
    let virt_dev = virt_dev.name("Remapped Trackball Mouse")
        .with_keys(&*info.mouse_buttons)?
        .with_relative_axes(&*info.axes)?
        .build()?;

    Ok(virt_dev)
}

// core loop for reading device events
async fn read_device(mut physical: Device, sender: Sender<InputEvent>) {
    loop {
        for event in physical.fetch_events().unwrap() {
            let _ = sender.send(event).await;
        }
    }
}

// core loop for writing processed device events
async fn write_virt_device(map: CmdMap, mut virt: VirtualDevice, mut receiver: Receiver<InputEvent>) {
    loop {
        let event = receiver.recv().await.unwrap();
        if ![evdev::EventType::RELATIVE, evdev::EventType::SYNCHRONIZATION].contains(
                &event.event_type()) {
            // translate the event correctly
            match map.translate_command(event) {
                Ok(mapped_event) => {
                    info!("EVENT: {:?} mapped to {:?} ({} to {})", 
                        evdev::Key(event.code()), 
                        evdev::Key(mapped_event.code()),
                        event.code(), 
                        mapped_event.code()
                    );
                    let _ = virt.emit(&[mapped_event]);
                },
                Err(_) => {
                    info!("EVENT: {:?} ({}) - {}", evdev::Key(event.code()), event.code(), event.value());
                    println!("No mapped event for {:?}", evdev::Key(event.code()));
                    let _ = virt.emit(&[event]);
                }
            }
        } else {
            let _ = virt.emit(&[event]);
        }
    }
}
