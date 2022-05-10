use std::{
    convert::AsMut,
    fmt,
    net::{Ipv4Addr, UdpSocket},
    str,
};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

fn challenge() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
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

struct ServerListSegments {
    header: Vec<u8>,
    command: Vec<u8>,
    servers: Vec<Server>,
    invalid: Vec<Vec<u8>>,
}

impl ServerListSegments {
    fn new() -> ServerListSegments {
        ServerListSegments {
            header: Vec::new(),
            command: Vec::new(),
            servers: Vec::new(),
            invalid: Vec::new(),
        }
    }
}

struct SendResult {
    error: bool,
    size: usize,
    buffer: [u8; 4096],
}

fn send(socket: &UdpSocket, packet: &[u8]) -> SendResult {
    let mut buffer = [0; 4096];
    let mut error = false;
    socket.send(packet).ok().expect("failed to send message");
    let (size, peer) = socket
        .recv_from(&mut buffer)
        .unwrap_or_else(|_| {
            return (
                0,
                std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)),
            );
        });

    if size == 0 {
        error = true;
    }

    SendResult {
        error,
        size,
        buffer,
    }
}

fn connect(address: &str) -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    socket.connect(address).unwrap();
    socket
}

fn get_servers() -> ServerListSegments {
    let socket = connect("master.xlabs.dev:20810");
    // header xFF xFF xFF xFF, command getservers, game IW4, protocol 150, full & empty seem to make no difference
    let packet = b"\xFF\xFF\xFF\xFFgetservers\nIW4 150 full empty";
    let response = send(&socket, packet);

    // Parse response
    // Response segments
    let mut segments = ServerListSegments::new();
    // Current segment
    let mut segment: Vec<u8> = Vec::new();
    // Total count of bytes in response
    let mut byte_count = 0;
    // Total count of segments in response
    let mut segment_count = 0;

    for b in &response.buffer {
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
                // println!(
                //     "Invalid segment at position {}: {:?}",
                //     segment_count, segment
                // );
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

    segments
}

struct Info {
    error: bool,
    text: String,
}

fn get_server_info(ip: Ipv4Addr, port: u16) -> Info {
    let socket = connect(&format!("{}:{}", ip, port));
    let packet = b"\xFF\xFF\xFF\xFFgetinfo";
    let response = send(&socket, packet);

    if response.error {
        return Info {
            error: true,
            text: "".to_string(),
        }
    }

    let mut byte_count = 0;
    let mut header = [0; 4];
    let mut info = Vec::new();
    let mut command: Vec<u8> = Vec::new();
    let mut command_found = false;

    for b in &response.buffer {
        if *b == 0xFF && byte_count < 4 {
            header[byte_count] = *b;
            byte_count += 1;
            continue;
        } else if *b != 0xFF && byte_count < 4 {
            println!("Invalid header byte {:02x} at position {}", b, byte_count);
            break;
        }

        if !command_found {
            if *b == 0x5c {
                command_found = true;
            }
            command.push(*b);
            continue;
        }

        info.push(*b);
        byte_count += 1;
    }

    
    Info {
        error: false,
        text: str::from_utf8(&info)
        .unwrap()
        .to_string()
        .trim()
        .to_string()
    }
}

fn main() {
    let segments = get_servers();
    println!("Servers: {}", segments.servers.len());
    println!("Header: {:?}", segments.header);
    println!("Command: {:?}", segments.command);

    for server in &segments.servers {
        let info = get_server_info(server.ip, server.port);
        if !info.error {
            println!("{}", info.text);
 
        }
    }
    // let info = get_server_info("51.195.206.173".parse().unwrap(), 28963);
    // println!("{}", info);
}
