use codecrafters_dns_server::Message;
use std::env;
use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    let args: Vec<String> = env::args().collect();

    println!("Args: {:?}", args);

    let resolver = if args.len() == 3 {
        println!("Using resolver {}", args[2]);
        Some(&args[2])
    } else {
        None
    };

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut response = Message::from_buf(&buf);

                if let Some(address) = resolver {
                    println!("Sending to resolver: {}", address);
                    let resolver_socket =
                        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind for resolver");

                    for question in response.get_questions() {
                        let mut req = Message::from_header_buf(&response.head);

                        req.push_question(&question);

                        resolver_socket
                            .send_to(&req.get_bytes(), address)
                            .expect("Failed to send to resolver");

                        let mut res_buf = [0u8; 512];

                        println!("Sent to resolver");

                        match resolver_socket.recv_from(&mut res_buf, ) {
                            Ok((size, source)) => {
                                println!("Got {} bytes from {}", size, source);
                                let message = Message::from_buf(&res_buf);

                                for ans in message.get_answers() {
                                    response.push_answer(ans);
                                }
                            },
                            Err(e) => {
                                println!("Failed to resolve: {}", e)
                            }
                        }
                    }
                } else {
                    for question in response.get_questions() {
                        response.add_answer(question, 2200, Ipv4Addr::new(8, 8, 8, 8))
                    }
                }

                response.qr(true);
                let opcode = if response.get_opcode() == 0u8 { 0 } else { 4u8 };
                response.set_rcode(opcode);

                udp_socket
                    .send_to(&response.get_bytes(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
