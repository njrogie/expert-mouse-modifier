use log::info;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::fs;
use evdev::{uinput::{VirtualDevice, VirtualDeviceBuilder}, InputEvent, Device};
use deviceinfo::DeviceInfo;

mod deviceinfo;

pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    // find the devices linked to kensington
    let device = find_devices()?[0].clone();
    let mut device = Device::open(device)?;
    
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

fn build_virtual_device(phys_device: &Device) -> Result<VirtualDevice, std::io::Error> {
    let info = DeviceInfo::new(&phys_device);
    let virt_dev = VirtualDeviceBuilder::new()?;
    let virt_dev = virt_dev.name("Remapped Trackball Mouse")
        .with_keys(info.mouse_buttons)?
        .with_relative_axes(info.axes)?
        .build()?;

    Ok(virt_dev)
}

async fn manage_events(device: Device) {
    info!("Creating virtual device...");

    let (tx, rx) = mpsc::channel(100);

    let virt_dev = build_virtual_device(&device).unwrap();

    // spawn a worker thread that receives 'processed' events
    tokio::spawn(async move {
        write_virt_device(virt_dev, rx).await;
    });

    read_device(device, tx).await;
}

async fn read_device(mut physical: Device, sender: Sender<Vec<InputEvent>>) {
    loop {
        let events = physical.fetch_events().unwrap();
        let mut input_vec:Vec<InputEvent> = vec![];
        for event in events {
            input_vec.push(event);
        }    
        let _ = sender.send(input_vec).await;
    }
}
async fn write_virt_device(mut virt: VirtualDevice, mut receiver: Receiver<Vec<InputEvent>>) {
    loop {
        let event_vec = receiver.recv().await.unwrap();
        let _ = virt.emit(&event_vec);
    }
}