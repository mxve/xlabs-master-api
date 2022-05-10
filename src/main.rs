use std::convert::AsMut;
use std::net::{Ipv4Addr, UdpSocket};
use std::fmt;
use std::str;

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

struct Server {
    ip: std::net::Ipv4Addr,
    port: u16
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

fn main() {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    socket.connect("master.xlabs.dev:20810").unwrap();
    socket.send(b"\xFF\xFF\xFF\xFFgetservers\nIW4 150 full empty").unwrap();
    let mut buffer = [0; 512];
    let len = socket.recv(&mut buffer).unwrap();

    // 0-4: header
    // 5-22: protocol getserversResponse
    let mut server_list: Vec<Server> = Vec::new();

    let mut count = 0;
    let mut parsed: Vec<u8> = Vec::new();
    let mut parts = 0;
    for b in &buffer[0..len] {

        // header
        if b == &255 && count < 4 {
            count += 1;
            continue;
        } else if b != &255 && count < 4 {
            println!("Invalid header byte {:02x} at position {}", b, count);
            break;
        }

        if *b == 0x5c {
            if parts == 0 {
                println!("Command: {:?}", str::from_utf8(&parsed).unwrap());
            } else if parsed.len() == 6 {
                let addrr: [u8; 4] = clone_into_array(&parsed[0..4]);
                let port: [u8; 2] = clone_into_array(&parsed[4..6]);
    
                let server = Server {
                    ip: Ipv4Addr::from(addrr),
                    port: u16::from_be_bytes(port)
                };
                server_list.push(server);
            } else {
                println!("Invalid server at position {}: {:?}", parts, parsed);
            }
            parsed = Vec::new();
            parts += 1;
        } else {
            parsed.push(*b);
        }
    }

    for server in server_list {
        println!("{:#}", server);
    }
}