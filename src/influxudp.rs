//! This contains the bare minimum needed to send data to influxdb through the wire protocol.
//!
//! The protocol is documented in the 
//! [InfluxDB Manual](https://docs.influxdata.com/influxdb/v1.5/write_protocols/line_protocol_reference/)

use std::net::UdpSocket;
use std::io::Result;

/// Represents a line in the influx wire protocol
pub struct WireLine<'a> {
    pub measurement: &'a str,
    pub field: &'a str,
    pub value: f32,
}

impl<'a> self::WireLine<'a> {
    /// Format the struct as line protocol string
    fn to_line(&self) -> String {
        format!("{} {}={}", self.measurement, self.field, self.value)
    }
}

pub fn send(socket: &UdpSocket, reading: WireLine ) -> Result<usize> {
    socket.send(reading.to_line().as_bytes())
}

