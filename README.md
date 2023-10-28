# RustyDHCP

![Rust](https://img.shields.io/badge/Language-Rust-orange)
![Dependencies](https://img.shields.io/badge/Dependencies-None-brightgreen)
![Crossplatform](https://img.shields.io/badge/Crossplatform-Yes-brightgreen)
![Cross Compilation](https://img.shields.io/badge/Cross%20Compilation-Supported-brightgreen)

A simple and zero-dependency DHCP server written in Rust, with credit to Richard Warburton for contributions to parts of the code.

## Features

- Lightweight and minimalistic DHCP server.
- Zero external dependencies; just Rust!
- Easy to use and configure.
- Based on reliable networking libraries.
- Fast and efficient.
- Cross-platform support and cross-compilation.
- Customizable Leases File: Support for a "leases" file that allows you to define permanent leases, ensuring clients always receive the same IP address.
  
## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Contributions](#contributions)
- [License](#license)

## Installation

1. Make sure you have Rust installed. If not, install it from [https://www.rust-lang.org/](https://www.rust-lang.org/).

2. Clone this repository:

   ```bash
   git clone https://github.com/pj1234678/RustyDHCP.git
   ```

3. Build the server:

   ```bash
   cd rusty-dhcp
   cargo build --release
   ```

## Usage

1. Start the DHCP server:

   ```bash
   sudo ./target/release/rusty-dhcp
   ```

   The server will listen on the default DHCP ports (67 and 68) and start serving DHCP requests.

2. Make DHCP requests from clients, and the server will respond with IP addresses and other configuration details.

## Configuration

To configure the server, edit the following fields in the `examples/server.rs` file:

```rust
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
```

You can customize these configuration parameters according to your network requirements.



To create a "leases" file with the example permanent lease, you can manually create a file named "leases" in the same directory as the compiled program with the following content:

f4:5c:19:af:96:8d,192.168.2.90

This lease format specifies the MAC address and the corresponding IP address for the client. The DHCP server will read this file to assign permanent leases based on its contents.


## Contributions

This DHCP server has been made possible with contributions from the open-source community, including valuable code from Richard Warburton. Feel free to contribute to this project and make it even better!

If you find a bug or have a feature request, please open an issue on the GitHub repository.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Note:** Remember to use this DHCP server responsibly and comply with local network regulations and security practices.
