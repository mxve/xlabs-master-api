use std::{
    convert::AsMut,
    fmt,
    net::{Ipv4Addr, UdpSocket},
    str,
};

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
    port: u16,
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

struct Segments {
    header: Vec<u8>,
    command: Vec<u8>,
    servers: Vec<Server>,
    invalid: Vec<Vec<u8>>,
}

impl Segments {
    fn new() -> Segments {
        Segments {
            header: Vec::new(),
            command: Vec::new(),
            servers: Vec::new(),
            invalid: Vec::new(),
        }
    }
}

fn main() {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    socket.connect("master.xlabs.dev:20810").unwrap();

    // Send request
    // header xFF xFF xFF xFF, command getservers, game IW4, protocol 150, params?
    socket
        .send(b"\xFF\xFF\xFF\xFFgetservers\nIW4 150 full empty")
        .unwrap();
    let mut buffer = [0; 4096];
    let len = socket.recv(&mut buffer).unwrap();

    // Parse response

    // Response segments
    let mut segments = Segments::new();
    // Current segment
    let mut segment: Vec<u8> = Vec::new();
    // Total count of bytes in response
    let mut byte_count = 0;
    // Total count of segments in response
    let mut segment_count = 0;

    for b in &buffer[0..len] {
        // Header segment
        // Valid header is 4 bytes long, which all are 0xFF (ÿ)
        if *b == 0xFF && byte_count < 4 {
            segments.header.push(*b);
            byte_count += 1;
            continue;
        } else if *b != 0xFF && byte_count < 4 {
            println!("Invalid header byte {:02x} at position {}", b, byte_count);
            break;
        }

        // Segments are separated by 0x5c (/)
        if *b == 0x5c {
            if segment_count == 0 {
                // First segment is command
                println!("Command: {:?}", str::from_utf8(&segment).unwrap());
                segments.command = segment;
            } else if segment.len() == 6 {
                // The following segments with length 6 are servers

                // First 4 bytes are the ip address
                let addrr: [u8; 4] = clone_into_array(&segment[0..4]);
                // Last 2 bytes are the port
                let port: [u8; 2] = clone_into_array(&segment[4..6]);

                let server = Server {
                    ip: Ipv4Addr::from(addrr),
                    port: u16::from_be_bytes(port),
                };
                segments.servers.push(server);
            } else {
                println!(
                    "Invalid segment at position {}: {:?}",
                    segment_count, segment
                );
                segments.invalid.push(segment);
            }

            // Reset segment
            segment = Vec::new();
            segment_count += 1;
        } else {
            segment.push(*b);
        }
        byte_count += 1;
    }

    // We do a little printing
    for server in &segments.servers {
        println!("{:#}", server);
    }

    println!("Servers: {}", segments.servers.len());
    println!("Header: {:?}", segments.header);
    println!("Command: {:?}", segments.command);
}
