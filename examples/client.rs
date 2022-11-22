use std::error::Error;
use std::net::UdpSocket;
use std::env;
use std::time::{Duration, Instant};

use quakeworld::protocol::message::Message;
use quakeworld::protocol::types::{Packet,ServerMessage};
use quakeworld::network::connection::{client, client::ClientConnectionState};
use quakeworld::utils::ascii_converter::AsciiConverter;

fn generate_crc(_map_name: String) -> u32 {
    return 1849737252; // maps/maphub_v1.bsp
}

fn connect(local_ip: String, remote_ip: String) -> Result<bool, Box<dyn Error>> {
    // initialize socket and client struct
    let ascii_converter = AsciiConverter::new();
    let socket = UdpSocket::bind(local_ip.clone())?;
    let mut buf = vec![0_u8; 1024 * 4];
    let s_a = socket.local_addr()?;
    let _ = socket.set_read_timeout(Some(Duration::new(2, 0)));
    let mut client = client::Client::new(s_a.to_string(), ascii_converter);

    let mut last_time = Instant::now();

    client.userinfo.update_from_string("name", "rust_user");
    client.userinfo.update_from_string("*client", "rust_quake");
    client.userinfo.update_from_string("spectator", "1");
    client.userinfo.update_from_string("rate", "25000");

    // generate and send the challenge package
    let get_challenge_packet = client.connect(s_a.port());
    let r = socket.send_to(&get_challenge_packet, &remote_ip)?;
    assert_eq!(r, get_challenge_packet.len());

    let mut count = 0;
    let mut connected = false;
    loop {
        // recieve server packet
        let mut message = Message::empty();
        message.bigendian = true;
        let server_packet = match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                buf[..n].to_vec()
            },
            Err(..) => {
                [].to_vec()
            },
        };

        // handle a timeout
        if server_packet.is_empty() {
            let status  = client.handle_timeout()?;
            if let Some(mut response) = status.response {
                let r = socket.send_to(&response, &remote_ip)?;
                assert_eq!(r, response.len());
            }
            continue;
        }

        // parse the packet
        let status = client.handle_packet(server_packet)?;
        // see if we need to send a response
        if let Some(mut response) = status.response {
            if last_time.elapsed().as_secs() > 10 {
                message.write_client_command_string(format!("say hello! from  rust for the {}... time", count));
                count += 1;
                response.extend(message.buffer.to_vec());
                last_time = Instant::now();
            }
            let r = socket.send_to(&response, &remote_ip)?;
            assert_eq!(r, response.len());
        }

        // reduce socket read timeout once we are connected
        if client.state == ClientConnectionState::Connected  && !connected {
            let _ = socket.set_read_timeout(Some(Duration::new(0, 200000000)));
            connected = true;
        }

        if let Some(p) = status.packet {
            match p {
                Packet::Connected(p) => {
                    for message in p.messages {
                        match message {
                            ServerMessage::Print(pr) => {
                                println!("{}", pr.message.string);
                                if pr.message.string.contains("rust panic!") {
                                    return Ok(false);
                                }
                            },
                            ServerMessage::Disconnect(..) => {
                                println!("we got diconnected :(");
                                return Ok(true);
                            },
                            ServerMessage::Modellist(modellist) => {
                                if modellist.start == 0 {
                                    client.map_crc = generate_crc(modellist.models[0].string.clone())
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a remote ip");
        return
    }
    let remote_ip = &args[1];
    match connect("0.0.0.0:0".to_string(), remote_ip.to_string()) {
        Ok(rv) => {
            if rv {
                println!("we got disconnected");
            } else {
                println!("we were told to panic");
            }
        }
        Err(err) => {
            eprintln!("could not connect to {}: {}", remote_ip, err);
        }
    }
}
