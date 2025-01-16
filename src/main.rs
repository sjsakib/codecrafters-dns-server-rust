use codecrafters_dns_server::packet::DnsPacket;
use std::env;
use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    let args: Vec<String> = env::args().collect();

    let resolver = if args.len() == 3 {
        Some(&args[2])
    } else {
        None
    };

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut packet = DnsPacket::from_buf(&buf);

                println!("Received packet: ");
                packet.print();

                // answer_packet(&mut packet, resolver);

                if let Some(address) = resolver {
                    let resolver_socket =
                        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind for resolver");

                    for question in packet.get_questions() {
                        let mut req = DnsPacket::new_query();

                        // req.set_rd(1);

                        req.push_question(question);

                        println!("Sending packet to resolver: {}", address);
                        req.print();

                        resolver_socket
                            .send_to(&req.get_bytes(), address)
                            .expect("Failed to send to resolver");

                        let mut res_buf = [0u8; 512];

                        match resolver_socket.recv_from(&mut res_buf) {
                            Ok((size, source)) => {
                                println!("Got {} bytes from {}", size, source);
                                let response = DnsPacket::from_buf(&res_buf);

                                response.print();

                                for ans in response.get_answers() {
                                    packet.push_answer(ans);
                                }
                            }
                            Err(e) => {
                                println!("Failed to resolve: {}", e)
                            }
                        }
                    }
                } else {
                    for question in packet.get_questions() {
                        packet.add_answer(question, 2200, Ipv4Addr::new(8, 8, 8, 8))
                    }
                }

                packet.set_qr(1);
                let rcode = if packet.get_opcode() == 0u8 { 0 } else { 4u8 };
                packet.set_rcode(rcode);

                println!("Answering");
                packet.print();

                udp_socket
                    .send_to(&packet.get_bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}


// fn answer_packet(packet: &mut DnsPacket, resolver: Option<&String>) {
//
// }

// fn answer_with_resolver()