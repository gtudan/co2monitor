extern crate hid;

use std::error::Error;
use hid::Manager;
use std::time::Duration;

const HID_SETREPORT: u32 = 0xc0094806;

fn main() {
    initialize();
}

fn initialize() {
    let manager = Manager;
    let mut devices = manager.find(Some(0x04d9), Some(0xa052));
    let device = match devices.next() {
        Some(device) => device,
        None => panic!("Device not found!"),
    };

    println!("Found device at address {}", device.path().display());

    let mut handle = match device.open() {
        Ok(handle) => handle,
        Err(why) => panic!("Failed to open device {}", why.description()),
    };

    let report_id: u8 = 0x1;
    let feature = [0x09];
    handle.feature().send_to(report_id, feature);

    let mut data: [u8; 8] = [0; 8];
    loop {
        match handle.data().read(&mut data, Duration::from_secs(5)) {
            Ok(length) => println!("Read {} bytes", length.expect("No bytes read!")),
            Err(why) => println!("{}", why.description()),
        }
        println!("{:?}", data);
    }
}
