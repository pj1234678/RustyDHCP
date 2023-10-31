use std::collections::HashMap;
use std::net::{Ipv4Addr, UdpSocket};
use std::ops::Add;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{BufRead, BufReader};

use dhcp4r::{options, packet, server};

// Server configuration
const SERVER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 2, 1);
const IP_START: [u8; 4] = [192, 168, 2, 2];
const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
const DNS_IPS: [Ipv4Addr; 1] = [
    // Google DNS servers
    Ipv4Addr::new(8, 8, 8, 8),
];
const ROUTER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 2, 1);
const BROADCAST_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 2, 255);
const LEASE_DURATION_SECS: u32 = 86400;
const LEASE_NUM: u32 = 252;

// Derived constants
const IP_START_NUM: u32 = u32::from_be_bytes(IP_START);
const INFINITE_LEASE: Option<Instant> = None; // Special value for infinite lease


fn main() {
    let socket = UdpSocket::bind("0.0.0.0:67").unwrap();
    socket.set_broadcast(true).unwrap();

  let mut leases: HashMap<Ipv4Addr, ([u8; 6], Option<Instant>)> = HashMap::new();
    // Read and populate leases from the file
if let Ok(file) = File::open("leases") {
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let mac_parts: Vec<u8> = parts[0]
                    .split(':')
                    .filter_map(|part| u8::from_str_radix(part, 16).ok())
                    .collect();

                if mac_parts.len() == 6 {
                    let mut mac = [0u8; 6];
                    mac.copy_from_slice(&mac_parts);

                    let ip = parts[1].trim().parse::<Ipv4Addr>().unwrap();
                    leases.insert(ip, (mac, INFINITE_LEASE));
                }
            }
        }
    }
} else {
    eprintln!("Failed to open leases file. Continuing...");
    //return;
}
    
    let ms = MyServer {
        leases,
        last_lease: 0,
        lease_duration: Duration::new(LEASE_DURATION_SECS as u64, 0),
    };

    server::Server::serve(socket, SERVER_IP, BROADCAST_IP, ms);
}

struct MyServer {
    leases: HashMap<Ipv4Addr, ([u8; 6], Option<Instant>)>, // Ipv4Addr -> (MAC address, lease duration) mapping
    last_lease: u32,
    lease_duration: Duration,
}

impl server::Handler for MyServer {
    fn handle_request(&mut self, server: &server::Server, in_packet: packet::Packet) {
        match in_packet.message_type() {
            Ok(options::MessageType::Discover) => {
		
                // Otherwise prefer existing (including expired if available)
                if let Some(ip) = self.current_lease(&in_packet.chaddr) {
			println!("Sending Reply to discover");
                    reply(server, options::MessageType::Offer, in_packet, &ip);
                    return;
                }
                // Otherwise choose a free ip if available
                for _ in 0..LEASE_NUM {
                    self.last_lease = (self.last_lease + 1) % LEASE_NUM;
                    if self.available(
                        &in_packet.chaddr,
                        &((IP_START_NUM + &self.last_lease).into()),
                    ) {
						println!("Sending Reply to discover");
                        reply(
                            server,
                            options::MessageType::Offer,
                            in_packet,
                            &((IP_START_NUM + &self.last_lease).into()),
                        );
                        break;
                    }
                }
            }

            Ok(options::MessageType::Request) => {
                // Ignore requests to alternative DHCP server
                if !server.for_this_server(&in_packet) {
					//println!("Not for this server");
                   // return;
                }
		
                let req_ip = match in_packet.option(options::REQUESTED_IP_ADDRESS) {
                    Some(options::DhcpOption::RequestedIpAddress(x)) => *x,
                    _ => in_packet.ciaddr,
                };
		 for (ip, (mac, _)) in &self.leases {
            println!("IP: {:?}, MAC: {:?}", ip, mac);
        }
		   if let Some(ip) = self.current_lease(&in_packet.chaddr) {
			println!("Found Current Lease");
                    reply(server, options::MessageType::Ack, in_packet, &ip);
                    return;
                }
                if !&self.available(&in_packet.chaddr, &req_ip) {
						println!("Sending Reply to Request");
                    nak(server, in_packet, "Requested IP not available");
                    return;
                }
                self.leases.insert(req_ip, (in_packet.chaddr, Some(Instant::now().add(self.lease_duration))));					
		println!("Sending Reply to Request");
                reply(server, options::MessageType::Ack, in_packet, &req_ip);
            }

            Ok(options::MessageType::Release) | Ok(options::MessageType::Decline) => {
                // Ignore requests to alternative DHCP server
                if !server.for_this_server(&in_packet) {
                    return;
                }
                if let Some(ip) = self.current_lease(&in_packet.chaddr) {
                    self.leases.remove(&ip);
                }
            }

            // TODO - not necessary but support for dhcp4r::INFORM might be nice
            _ => {}
        }
    }
}

impl MyServer {
    fn available(&self, chaddr: &[u8; 6], addr: &Ipv4Addr) -> bool {
        let pos: u32 = (*addr).into();
        pos >= IP_START_NUM
            && pos < IP_START_NUM + LEASE_NUM
            && match self.leases.get(addr) {
                Some((mac, expiry)) => {
                    *mac == *chaddr || expiry.map_or(true, |exp| Instant::now().gt(&exp))
                }
                None => true,
            }
    }
	    fn current_lease(&self, chaddr: &[u8; 6]) -> Option<Ipv4Addr> {

        for (i, v) in &self.leases {
            if v.0 == *chaddr {
                return Some(*i);
            }
        }
        None
    }
}

fn reply(
    s: &server::Server,
    msg_type: options::MessageType,
    req_packet: packet::Packet,
    offer_ip: &Ipv4Addr,
) {
    let _ = s.reply(
        msg_type,
        vec![
            options::DhcpOption::IpAddressLeaseTime(LEASE_DURATION_SECS),
            options::DhcpOption::SubnetMask(SUBNET_MASK),
            options::DhcpOption::Router(vec![ROUTER_IP]),
            options::DhcpOption::DomainNameServer(DNS_IPS.to_vec()),
        ],
        *offer_ip,
        req_packet,
    );
}

fn nak(s: &server::Server, req_packet: packet::Packet, message: &str) {
    let _ = s.reply(
        options::MessageType::Nak,
        vec![options::DhcpOption::Message(message.to_string())],
        Ipv4Addr::new(0, 0, 0, 0),
        req_packet,
    );
}
