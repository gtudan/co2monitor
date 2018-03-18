//! A module for reading temperatue and co2 concentration from a Dostmann CO2-Sensor
extern crate hidapi;

#[derive(PartialEq, Debug)]
pub enum Reading {
    /// a temperature reading in degrees of celcius
    Temperature(f32),
    /// a co2 concentration in parts-per-million (PPM) 
    CO2(u16),
}

/// Readings are encoded with a key - I'll just set a static one
const KEY: [u8; 8] = [0xc4, 0xc6, 0xc0, 0x92, 0x40, 0x23, 0xdc, 0x96];

/// Detect the co2 reader and set it into reading mode
///
/// # Panics
///
/// This will fail if the device is not connected or if you are missing
/// the required permissions for writing to the device
pub fn initialize<'a>(api: &'a hidapi::HidApi) -> hidapi::HidDevice<'a> {
    let device = api.open(0x04d9, 0xa052).expect("Device not found!");

    println!("Found device");

    let report_id = 0x00;
    let mut buffer = Vec::with_capacity(KEY.len() + 1);
	buffer.push(report_id);
    buffer.extend(KEY.to_vec());
    device.send_feature_report(&buffer).ok();
    device
}

/// Get a reading from the device. The device must be in reading mode.
pub fn read_data(device: &mut hidapi::HidDevice) -> Reading {
    let mut data = [0u8; 8];
    device.read(&mut data).ok();
    let decrypted = decrypt(data);
    validate_checksum(&decrypted).ok();
    
    match decode(decrypted) {
        Some(value) => value,
        None => read_data(device),
    }
}

/// Decrypt a reading to get the raw data.
///
/// This decrypt logic was discovered by Henry Plötz and documented 
/// [here](https://hackaday.io/project/5301/logs)
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

/// Validate the checksum of a packet
///
/// # Errors
///
/// This will return an error if the package was corrupted
fn validate_checksum(decrypted: &[u8; 8]) -> Result<(), &'static str> {
    let sum: u8 = (decrypted[0..3].iter().map(|x| *x as u16).sum::<u16>() & 0xff) as u8;
    if decrypted[4] != 0x0d || sum != decrypted[3] {
        return Err("Checksum validation failed");
    } else {
        return Ok(());
    }
}

/// Decodes the readings to co2 concentration and temperature. This is documented 
/// [here](http://co2meters.com/Documentation/AppNotes/AN146-RAD-0401-serial-communication.pdf)
///
/// There are return values that I could not make sens of. These are returned as `None`.
fn decode(decrypted: [u8; 8]) -> Option<Reading> {
    let op = decrypted[0];
    let val = (decrypted[1] as u16) << 8 | decrypted[2] as u16;

    match op {
        0x50 => Some(Reading::CO2(val)),
        0x42 => Some(Reading::Temperature(val as f32 / 16.0 - 273.15)),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn it_validates_checksum() {
        assert!(validate_checksum(&[0x50, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00]).is_ok());
        assert!(validate_checksum(&[0x51, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00]).is_err());
    }

    #[test]
    fn it_decodes_co2() {
        assert_eq!(Reading::CO2(1013),
                   decode([0x50, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00]).unwrap());
        assert_eq!(None, 
                   decode([0x51, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00]));
    }

    #[test]
    fn decode_ignores_unknown_values() {
        assert_eq!(None, decode([0x51, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00]));
    }
}
