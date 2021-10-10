//! A crate for reading co2 concentration from a Dostmann CO2-Sensor
//! and writing it to influxdb.
extern crate hidapi;
extern crate rumqttc;

use zytemp::Reading;
use std::thread;
use rumqttc::{QoS, MqttOptions, Client, Connection};

mod zytemp;

fn main() {
    let (client, mut connection) = connect_client();

    thread::spawn(move || publish_readings(client) );

    for (i, notification) in connection.iter().enumerate() {
        //println!("{}. Notification {:?}", i, notification)
    }
}

fn connect_client() -> (Client, Connection) {
    let mut options = MqttOptions::new("co2monitor", "homebridge.local", 1883);
    options.set_keep_alive(5);
    options.set_clean_session(true);
    Client::new(options, 10)
}

fn publish_readings(client: Client) -> ! {
    let api: hidapi::HidApi = hidapi::HidApi::new().unwrap();
    let mut device = zytemp::initialize(&api);

    loop {
        let reading = zytemp::read_data(&mut device);
        println!("{:?}", reading);

        let mut client_sender = client.clone();

        let result = match reading {
            Reading::CO2(v) => client_sender.publish("wohnzimmer/co2monitor/co2", QoS::AtLeastOnce, false, v.to_string().as_bytes()),
            Reading::Temperature(v) => client_sender.publish("wohnzimmer/co2monitor/temperature", QoS::AtLeastOnce, false, v.to_string().as_bytes()),
        };
        result.unwrap();
    }
}
