use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::{
    fmt,
    net::{Ipv4Addr, UdpSocket},
    str,
};

#[path = "utils.rs"]
mod utils;

pub struct Server {
    pub ip: std::net::Ipv4Addr,
    pub port: u16,
}

pub struct ServerListSegments {
    pub header: Vec<u8>,
    pub command: Vec<u8>,
    pub servers: Vec<Server>,
    pub invalid: Vec<Vec<u8>>,
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

pub struct FullServer {
    pub game: String,
    pub ip: std::net::Ipv4Addr,
    pub port: u16,
    pub cod_info: String,
}

impl Serialize for FullServer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Server", 4)?;
        state.serialize_field("game", &self.game)?;
        state.serialize_field("ip", &self.ip)?;
        state.serialize_field("port", &self.port)?;
        state.serialize_field("codInfo", &self.cod_info)?;
        state.end()
    }
}

pub struct SendResult {
    pub error: bool,
    pub size: usize,
    pub buffer: [u8; 4096],
}

pub struct Info {
    pub error: bool,
    pub text: String,
}

pub enum Game {
    IW4,
    IW6,
    S1,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Game::IW4 => write!(f, "iw4x"),
            Game::IW6 => write!(f, "iw6x"),
            Game::S1 => write!(f, "s1x"),
        }
    }
}

pub fn send(socket: &UdpSocket, packet: &[u8]) -> SendResult {
    let mut buffer = [0; 4096];
    let mut error = false;
    socket.send(packet).expect("failed to send message");
    let (size, _peer) = socket.recv_from(&mut buffer).unwrap_or_else(|_| {
        (
            0,
            std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)),
        )
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

pub fn connect(address: &str) -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(300)))
        .unwrap();
    socket.connect(address).unwrap();
    socket
}

pub fn get_servers(game: Game) -> ServerListSegments {
    // dpmaster "getservers"
    let dp_command = "getservers\n".as_bytes();

    // game, protocol
    let game_info = match game {
        Game::IW4 => "IW4 150",
        Game::IW6 => "IW6 1",
        Game::S1 => "S1 1",
    }
    .as_bytes();

    let filters = " full empty".as_bytes();

    // combine command and game info
    let mut command = Vec::new();
    command.extend_from_slice(dp_command);
    command.extend_from_slice(game_info);
    command.extend_from_slice(filters);

    // build packet
    // first 4 bytes are header 0xff
    let mut packet_buffer = [0; 64];
    for byte in packet_buffer.iter_mut().take(4) {
        *byte = 0xff;
    }

    // append command bytes
    for i in 4..command.len() {
        packet_buffer[i] = command[i - 4].to_be_bytes()[0];
    }

    // send packet
    let socket = connect("master.xlabs.dev:20810");
    let response = send(&socket, &packet_buffer);

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
                let addrr: [u8; 4] = utils::clone_into_array(&segment[0..4]);
                // Last 2 bytes are the port
                let port: [u8; 2] = utils::clone_into_array(&segment[4..6]);

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

pub fn get_server_info(ip: Ipv4Addr, port: u16) -> Info {
    let dpcommand = "getinfo ".as_bytes();

    let challenge = utils::random_string(12);

    // combine command and challenge
    let mut command = Vec::new();
    command.extend_from_slice(dpcommand);
    command.extend_from_slice(challenge.as_bytes());

    // build packet
    // first 4 bytes are header 0xff

    // TODO
    // last 4 characters of challenge are returned as u0000
    // thus the packet_buffer is 4 bytes shorter, cutting those 4 chars
    let mut packet_buffer = [0; 20];
    for byte in packet_buffer.iter_mut().take(4) {
        *byte = 0xff;
    }

    // append command bytes
    for i in 4..command.len() {
        packet_buffer[i] = command[i - 4].to_be_bytes()[0];
    }

    let socket = connect(&format!("{}:{}", ip, port));
    let response = send(&socket, &packet_buffer);

    if response.error {
        return Info {
            error: true,
            text: "".to_string(),
        };
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
        text: String::from_utf8(info)
            .unwrap()
            .trim_matches(char::from(0))
            .to_string(),
    }
}

pub fn get_servers_full(game: Game) -> Vec<FullServer> {
    let mut servers = Vec::new();
    let game_name = &game.to_string();
    let master_servers = get_servers(game);

    for server in master_servers.servers {
        let info = get_server_info(server.ip, server.port);
        if !info.error {
            let full_server = FullServer {
                game: game_name.to_string(),
                ip: server.ip,
                port: server.port,
                cod_info: info.text.to_string(),
            };
            servers.push(full_server);
        }
    }

    servers
}
