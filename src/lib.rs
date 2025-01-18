use packet::{Answer, DnsPacket, Question};
use std::net::{Ipv4Addr, UdpSocket};

mod buffer_helpers;
pub mod packet;

pub struct ServerConfig {
    pub port: u16,
    pub resolver: Option<String>,
}

pub struct DnsServer {
    udp_socket: UdpSocket,
    resolver_address: Option<String>,
}

impl DnsServer {
    pub fn init(config: ServerConfig) -> DnsServer {
        println!("Starting server on port {}", config.port);
        let udp_socket = UdpSocket::bind(format!("127.0.0.1:{}", config.port)).expect("Failed to bind to address");

        DnsServer {
            udp_socket,
            resolver_address: config.resolver,
        }
    }

    pub fn listen(&self) {
        println!("Listening for requests");
        loop {
            let mut buf = [0; 512];
            match self.udp_socket.recv_from(&mut buf) {
                Ok((size, source)) => {
                    println!("Received {} bytes from {}", size, source);
                    let mut request = DnsPacket::from_buf(&buf);

                    println!("Received packet: ");
                    request.print();

                    let mut response = self.get_response(&mut request);

                    println!("Sending response:");
                    response.print();
                    println!("\n");

                    self.udp_socket
                        .send_to(&response.get_bytes(), source)
                        .expect("Failed to send response");
                }
                Err(e) => {
                    eprintln!("Failed to receive from socket: {}", e);
                }
            }
        }
    }

    fn get_response(&self, request: &mut DnsPacket) -> DnsPacket {
        let mut response = DnsPacket::new();

        response.set_id(request.get_id());
        response.set_qr(1);
        response.set_rd(request.get_rd());
        response.set_opcode(request.get_opcode());

        let opcode = request.get_opcode();

        if opcode != 0 {
            response.set_rcode(4);

            return response;
        }

        for question in request.get_questions() {
            let answers = self.get_answers(question.clone());

            match answers {
                Ok(answers) => {
                    response.push_question(question.clone());
                    for answer in answers {
                        response.push_answer(answer);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get answers: {}", e);

                    response.set_rcode(2);

                    break;
                }
            }
        }

        return response;
    }

    fn get_answers(&self, question: Question) -> Result<Vec<Answer>, String> {
        if let Some(resolver_address) = &self.resolver_address {
            let mut req = DnsPacket::new_query();

            req.push_question(question);

            req.set_rd(1);

            let resolver_socket =
                UdpSocket::bind("0.0.0.0:0").expect("Failed to bind for resolver");

            resolver_socket
                .send_to(&req.get_bytes(), resolver_address)
                .expect("Failed to send to resolver");

            let mut res_buf = [0u8; 512];

            match resolver_socket.recv_from(&mut res_buf) {
                Ok((size, _)) => {
                    println!("Received {} bytes from resolver", size);
                    let response = DnsPacket::from_buf(&res_buf);
                    response.print();

                    Ok(response.get_answers())
                }
                Err(e) => Err(format!("Failed to receive from resolver: {}", e)),
            }
        } else {
            Ok(vec![Answer {
                question,
                ttl: 2200,
                data: Ipv4Addr::new(8, 8, 8, 8).to_bits(),
            }])
        }
    }
}
