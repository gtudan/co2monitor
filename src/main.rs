//! A crate for reading co2 concentration from a Dostmann CO2-Sensor
//! and writing it to influxdb.
mod zytemp;
mod influxudp;

use std::net::UdpSocket;
use zytemp::Reading;
use influxudp::WireLine;

const INFLUXDB_UDP: &'static str = "127.0.0.1:8089";

fn main() {
    let mut device = zytemp::initialize();

    // Open an UDP-Socket to InfluxDB
    let socket = UdpSocket::bind("127.0.0.1:34567").expect("couldn't bind to address");
    socket.connect(INFLUXDB_UDP).expect("connect function failed");

    loop {
        let reading = zytemp::read_data(&mut device);
        println!("{:?}", reading);
        send_to_influxdb(&socket, reading);
    }
}

/// Send the reading to influxdb via UDP
///
/// This is pretty minimalistic and supports no security whatsoever, so this
/// is only a sane thing to do if influx is running on the same host 
fn send_to_influxdb(socket: &UdpSocket, reading: Reading) {
    let field;
    let value;

    match reading {
        Reading::CO2(v) => { field="CO2"; value=v as f32; },
        Reading::Temperature(v) => { field="temperature"; value=v; },
    }

    let line = WireLine {measurement: "climate", field: field, value: value };
    if influxudp::send(&socket, line).is_err() {
        println!("Failed to send measurement to Influx");
    }
}
