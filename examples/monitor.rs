extern crate dhcp4r;
extern crate time;

use dhcp4r::{options, packet, server};
use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    server::Server::serve(
        UdpSocket::bind("0.0.0.0:67").unwrap(),
        Ipv4Addr::new(0, 0, 0, 0),
        MyServer {},
    );
}

struct MyServer {}

impl server::Handler for MyServer {
    fn handle_request(&mut self, _: &server::Server, in_packet: packet::Packet) {
        match in_packet.message_type() {
            Ok(options::MessageType::Request) => {
                let req_ip = match in_packet.option(options::REQUESTED_IP_ADDRESS) {
                    Some(options::DhcpOption::RequestedIpAddress(x)) => x.clone(),
                    _ => in_packet.ciaddr,
                };
                println!(
                    "{}\t{}\t{}\tOnline",
                    time::OffsetDateTime::now_local().format("%Y-%m-%dT%H:%M:%S"),
                    chaddr(&in_packet.chaddr),
                    Ipv4Addr::from(req_ip)
                );
            }
            _ => {}
        }
    }
}

/// Formats byte array machine address into hex pairs separated by colons.
/// Array must be at least one byte long.
fn chaddr(a: &[u8]) -> String {
    a[1..].iter().fold(format!("{:02x}", a[0]), |acc, &b| {
        format!("{}:{:02x}", acc, &b)
    })
}
