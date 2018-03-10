extern crate hid;

use std::error::Error;
use std::time::Duration;

enum ValueType {
    Temperature,
    CO2,
}

struct Reading {
    pub value: u8,
    pub value_type: ValueType, 
} 

const KEY: [u8; 8] = [0xc4, 0xc6, 0xc0, 0x92, 0x40, 0x23, 0xdc, 0x96];

pub fn initialize() -> hid::Handle {
    let manager = hid::Manager;
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

    let report_id = 0x00;
    handle.feature().send_to(report_id, KEY).ok();
    handle
}

pub fn read_data(mut handle: hid::Handle) {
    let mut data = [0u8; 8];
    loop {
        handle.data().read(&mut data, Duration::from_secs(30)).ok();
        let decrypted = decrypt(data);
        validate_checksum(&decrypted).ok();
        decode(decrypted);
    }
}

#[test]
fn it_decrypts() {
    assert_eq!(
        [0x50, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00],
        decrypt([0x98, 0xE4, 0x66, 0x20, 0x94, 0x46, 0xBF, 0x62])
    );
    assert_eq!(
        [0x6E, 0x60, 0xA0, 0x6E, 0x0D, 0x0, 0x0, 0x0],
        decrypt([0x72, 0xE4, 0x51, 0x21, 0xF9, 0x46, 0xBF, 0xB2])
    );
}

fn decrypt(data: [u8; 8]) -> [u8; 8] {
    const CSTATE: [u8; 8] = [0x48, 0x74, 0x65, 0x6D, 0x70, 0x39, 0x39, 0x65];
    const SHUFFLE: [usize; 8] = [2, 4, 0, 7, 1, 6, 5, 3];

    let mut phase1 = [0; 8];
    for (i, &o) in SHUFFLE.iter().enumerate() {
        phase1[o] = data[i];
    }

    let mut phase2 = [0; 8];
    for i in 0..8 {
        phase2[i] = phase1[i] ^ KEY[i];
    }

    let mut phase3 = [0; 8];
    for i in 0..8 {
        phase3[i] = (phase2[i] >> 3 | phase2[(i + 7) % 8] << 5) & 0xff;
    }

    let mut tmp = [0; 8];
    for i in 0..8 {
        tmp[i] = (CSTATE[i] >> 4 | CSTATE[i] << 4) & 0xff;
    }

    let mut out = [0u8; 8];
    for i in 0..8 {
        out[i] = ((0x100u32 + u32::from(phase3[i]) - u32::from(tmp[i])) & 0xff) as u8;
    }

    out
}

fn validate_checksum(decrypted: &[u8; 8]) -> Result<(), &'static str> {
    let sum: u8 = (decrypted[0..3].iter().map(|x| *x as u16).sum::<u16>() & 0xff) as u8;
    if decrypted[4] != 0x0d || sum != decrypted[3] {
        return Err("Checksum validation failed");
    } else {
        return Ok(());
    }
}

fn decode(decrypted: [u8; 8]) {
    let op = decrypted[0];
    let val = (decrypted[1] as u16) << 8 | decrypted[2] as u16;

    // From http://co2meters.com/Documentation/AppNotes/AN146-RAD-0401-serial-communication.pdf
    if 0x50 == op {
        println!("CO2: {}", val);
    }
    if 0x42 == op {
        println!("T: {:2.2}", (val as f32 / 16.0 - 273.15));
    }
}
