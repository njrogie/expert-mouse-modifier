use log::info;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::{env, fs};
use evdev::{uinput::{VirtualDevice, VirtualDeviceBuilder}, Device, InputEvent};

mod eventprocessor;

use deviceinfo::DeviceInfo;
use eventprocessor::CmdMap;

mod deviceinfo;

// Create and run the middleman routine for a Kensington device
pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    // find the devices linked to kensington
    let device = find_devices()?[0].clone();
    let mut device = Device::open(device)?;
    
    let args: Vec<String> = env::args().collect();
    let cmd_map = match args.get(1) {
        Some(arg) => CmdMap::new(arg.clone()),
        None => CmdMap::default(), 
    };
    
    info!("Grabbing device {}", device.name().unwrap());
    let info = DeviceInfo::new(&device);    
    info!("{}", info);
    
    // grab the device and begin event transcription
    match device.grab() {
        Ok(_) => {
            manage_events(device).await;
            return Ok(())
        },
        Err(e) => {
            return Err(Box::new(e));
        }
    }
}

// Filter devices for the ones we want ("kensington" + "event-mouse" in input devices)
fn find_devices() -> Result<Vec<String>,Box<dyn std::error::Error>> {
    let devices = fs::read_dir("/dev/input/by-id")?;
    let mut kensington_devices: Vec<String> = vec![];

    // Select the kensington mouse device
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

// Create the virtual device and begin processing events.
async fn manage_events(device: Device) {
    let (tx, rx) = mpsc::channel(100);
    let virt_dev = build_virtual_device(&device).unwrap();

    // spawn a worker thread that receives 'processed' events
    tokio::spawn(async move {
        write_virt_device(virt_dev, rx).await;
    });

    // run the read on the main thread
    read_device(device, tx).await;
}

// Build the virtual device that's going to receive processed events.
fn build_virtual_device(phys_device: &Device) -> Result<VirtualDevice, std::io::Error> {
    let info = DeviceInfo::new(&phys_device);
    let virt_dev = VirtualDeviceBuilder::new()?;
    let virt_dev = virt_dev.name("Remapped Trackball Mouse")
        .with_keys(info.mouse_buttons)?
        .with_relative_axes(info.axes)?
        .build()?;

    Ok(virt_dev)
}

async fn read_device(mut physical: Device, sender: Sender<InputEvent>) {
    loop {
        for event in physical.fetch_events().unwrap() {
            let _ = sender.send(event).await;
        }
    }
}

async fn write_virt_device(mut virt: VirtualDevice, mut receiver: Receiver<InputEvent>) {
    loop {
        let event = receiver.recv().await.unwrap();

        info!("EVENT: {:?} ({}) - {}", evdev::Key(event.code()), event.code(), event.value());
        let _ = virt.emit(&[event]);
    }
}
