use crate::options::*;

use std::net::Ipv4Addr;

pub enum CustomErr<I> {
    NomError((I, ErrorKind)),
    NonUtf8String,
    UnrecognizedMessageType,
    InvalidHlen,
}

pub enum ErrorKind {
    Tag,
    MapRes,
    ManyTill,
    Eof,
    Custom(u32),
}

type IResult<I, O> = Result<(I, O), CustomErr<I>>;

/// DHCP Packet Structure
#[derive(Debug)]
pub struct Packet {
    pub reply: bool, // false = request, true = reply
    pub hops: u8,
    pub xid: u32, // Random identifier
    pub secs: u16,
    pub broadcast: bool,
    pub ciaddr: Ipv4Addr,
    pub yiaddr: Ipv4Addr,
    pub siaddr: Ipv4Addr,
    pub giaddr: Ipv4Addr,
    pub chaddr: [u8; 6],
    pub options: Vec<DhcpOption>,
}

fn decode_reply(input: &[u8]) -> IResult<&[u8], bool> {
    let (input, reply) = custom_take(1usize)(input)?;
    Ok((
        input,
        match reply[0] {
            BOOT_REPLY => true,
            BOOT_REQUEST => false,
            _ => {
                // @TODO: Throw an error
                false
            }
        },
    ))
}

fn decode_ipv4(p: &[u8]) -> IResult<&[u8], Ipv4Addr> {
    let (input, addr) = custom_take(4usize)(p)?;
    Ok((input, Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3])))
}
fn custom_many0<I, O, F>(mut f: F) -> impl FnMut(I) -> IResult<I, Vec<O>>
where
    I: Clone + PartialEq,
    F: FnMut(I) -> IResult<I, O>,
{
    move |input: I| {
        let mut acc = Vec::new();
        let mut remaining = input.clone();

        loop {
            match f(remaining.clone()) {
                Ok((input, o)) => {
                    if input == remaining {
                        return Ok((input, acc));
                    }
                    acc.push(o);
                    remaining = input;
                }
                Err(CustomErr::NomError(_)) => return Ok((remaining, acc)),
                Err(e) => return Err(e),
            }
        }
    }
}
pub fn decode_option(input: &[u8]) -> IResult<&[u8], DhcpOption> {
    let (input, code) = custom_be_u8(input)?;
    assert!(code != END);

    let (input, len) = custom_be_u8(input)?;
    let (input, data) = custom_take(len.into())(input)?;
    let option = match code {
        DHCP_MESSAGE_TYPE => {
            DhcpOption::DhcpMessageType(match MessageType::from(custom_be_u8(data)?.1) {
                Ok(x) => x,
                Err(_) => return Err(CustomErr::UnrecognizedMessageType),
            })
        }
        SERVER_IDENTIFIER => DhcpOption::ServerIdentifier(decode_ipv4(data)?.1),
        PARAMETER_REQUEST_LIST => DhcpOption::ParameterRequestList(data.to_vec()),
        REQUESTED_IP_ADDRESS => DhcpOption::RequestedIpAddress(decode_ipv4(data)?.1),
        HOST_NAME => DhcpOption::HostName(match std::str::from_utf8(data) {
            Ok(s) => s.to_string(),
            Err(_) => return Err(CustomErr::NonUtf8String),
        }),
        ROUTER => DhcpOption::Router(custom_many0(decode_ipv4)(data)?.1),
        DOMAIN_NAME_SERVER => DhcpOption::DomainNameServer(custom_many0(decode_ipv4)(data)?.1),
        IP_ADDRESS_LEASE_TIME => DhcpOption::IpAddressLeaseTime(custom_be_u32(data)?.1),
        SUBNET_MASK => DhcpOption::SubnetMask(decode_ipv4(data)?.1),
        MESSAGE => DhcpOption::Message(match std::str::from_utf8(data) {
            Ok(s) => s.to_string(),
            Err(_) => return Err(CustomErr::NonUtf8String),
        }),
        _ => DhcpOption::Unrecognized(RawDhcpOption {
            code,
            data: data.to_vec(),
        }),
    };
    Ok((input, option))
}
fn custom_take<'a>(n: usize) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    move |input: &'a [u8]| {
        if input.len() >= n {
            Ok((&input[n..], &input[0..n]))
        } else {
            Err(CustomErr::InvalidHlen)
        }
    }
}

fn custom_tag<'a>(tag: &'static [u8]) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    move |input: &'a [u8]| {
        if input.starts_with(tag) {
            Ok((&input[tag.len()..], &input[..tag.len()]))
        } else {
            Err(CustomErr::NomError((input, ErrorKind::Tag)))
        }
    }
}
fn custom_be_u8(input: &[u8]) -> IResult<&[u8], u8> {
    if input.is_empty() {
        return Err(CustomErr::InvalidHlen);
    }

    Ok((&input[1..], input[0]))
}

fn custom_be_u16(input: &[u8]) -> IResult<&[u8], u16> {
    if input.len() < 2 {
        return Err(CustomErr::InvalidHlen);
    }

    let value = u16::from_be_bytes([input[0], input[1]]);
    Ok((&input[2..], value))
}
fn custom_be_u32(input: &[u8]) -> IResult<&[u8], u32> {
    if input.len() < 4 {
        return Err(CustomErr::InvalidHlen);
    }

    let value = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    Ok((&input[4..], value))
}

/// Parses Packet from byte array
fn decode(input: &[u8]) -> IResult<&[u8], Packet> {
    let (options_input, input) = custom_take(236usize)(input)?;

    let (input, reply) = decode_reply(input)?;
    let (input, _htype) = custom_take(1usize)(input)?;
    let (input, hlen) = custom_be_u8(input)?;
    let (input, hops) = custom_be_u8(input)?;
    let (input, xid) = custom_be_u32(input)?;
    let (input, secs) = custom_be_u16(input)?;
    let (input, flags) = custom_be_u16(input)?;
    let (input, ciaddr) = decode_ipv4(input)?;
    let (input, yiaddr) = decode_ipv4(input)?;
    let (input, siaddr) = decode_ipv4(input)?;
    let (input, giaddr) = decode_ipv4(input)?;

    if hlen != 6 {
        return Err(CustomErr::InvalidHlen);
    }
    let (_, chaddr) = custom_take(6usize)(input)?;

    let input = options_input;
    let (input, _) = custom_tag(&COOKIE)(input)?;

    let mut options = Vec::new();
    let mut rest = input;

    while let Ok((new_rest, option)) = decode_option(rest) {
        rest = new_rest;
        options.push(option);
        if rest.starts_with(&[END]) {
            break;
        }
    }

    let input = rest.split_at(1).1; // Skip the END tag byte

    Ok((
        input,
        Packet {
            reply,
            hops,
            secs,
            broadcast: flags & 128 == 128,
            ciaddr,
            yiaddr,
            siaddr,
            giaddr,
            options,
            chaddr: [
                chaddr[0], chaddr[1], chaddr[2], chaddr[3], chaddr[4], chaddr[5],
            ],
            xid,
        },
    ))
}

impl Packet {
    pub fn from(input: &[u8]) -> Result<Packet, CustomErr<&[u8]>> {
        Ok(decode(input)?.1)
    }

    /// Extracts requested option payload from packet if available
    pub fn option(&self, code: u8) -> Option<&DhcpOption> {
        self.options.iter().find(|&option| option.code() == code)
    }

    /// Convenience function for extracting a packet's message type.
    pub fn message_type(&self) -> Result<MessageType, String> {
        match self.option(DHCP_MESSAGE_TYPE) {
            Some(DhcpOption::DhcpMessageType(msgtype)) => Ok(*msgtype),
            Some(option) => Err(format![
                "Got wrong enum code {} for DHCP_MESSAGE_TYPE",
                option.code()
            ]),
            None => Err("Packet does not have MessageType option".to_string()),
        }
    }
    pub fn encode<'a>(&'a self, p: &'a mut [u8]) -> &[u8] {
        let broadcast_flag = if self.broadcast { 128 } else { 0 };
        let mut length = 240;

        p[..12].copy_from_slice(&[
            if self.reply { BOOT_REPLY } else { BOOT_REQUEST },
            1,
            6,
            self.hops,
            ((self.xid >> 24) & 0xFF) as u8,
            ((self.xid >> 16) & 0xFF) as u8,
            ((self.xid >> 8) & 0xFF) as u8,
            (self.xid & 0xFF) as u8,
            (self.secs >> 8) as u8,
            (self.secs & 255) as u8,
            broadcast_flag,
            0,
        ]);

        p[12..16].copy_from_slice(&self.ciaddr.octets());
        p[16..20].copy_from_slice(&self.yiaddr.octets());
        p[20..24].copy_from_slice(&self.siaddr.octets());
        p[24..28].copy_from_slice(&self.giaddr.octets());
        p[28..34].copy_from_slice(&self.chaddr);
        p[34..236].fill(0);
        p[236..240].copy_from_slice(&COOKIE);

        for option in &self.options {
            let option = option.to_raw();
            let option_len = option.data.len();
            if length + 2 + option_len >= 272 {
                break;
            }
            if let Some(dest) = p.get_mut(length..length + 2 + option_len) {
                dest[0] = option.code;
                dest[1] = option_len as u8;
                dest[2..].copy_from_slice(&option.data);
            }
            length += 2 + option_len;
        }

        if let Some(end_segment) = p.get_mut(length..length + 1) {
            end_segment[0] = END;
        }
        length += 1;

        if let Some(pad_segment) = p.get_mut(length..272) {
            pad_segment.fill(PAD);
        }

        &p[..length]
    }
}

const COOKIE: [u8; 4] = [99, 130, 83, 99];

const BOOT_REQUEST: u8 = 1; // From Client;
const BOOT_REPLY: u8 = 2; // From Server;

const END: u8 = 255;
const PAD: u8 = 0;
