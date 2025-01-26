use packet::{Answer, DnsPacket, Question, ResponseCodes};
use std::{fmt::Display, net::UdpSocket, time::Duration};

mod buffer_helpers;
pub mod packet;

pub struct ServerConfig {
    pub port: u16,
    pub resolver: Option<String>,
}

pub struct DnsServer {
    config: ServerConfig,
}

#[derive(Debug)]
enum ServerError {
    Resolver(String),
    IO(String),
    // Unknown(String),
}

impl std::error::Error for ServerError {}

impl From<std::io::Error> for ServerError {
    fn from(other: std::io::Error) -> Self {
        ServerError::IO(format!("{}", other))
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            // ServerError::Unknown(msg) => msg,
            ServerError::IO(msg) => msg,
            ServerError::Resolver(msg) => msg,
        };
        write!(f, "{}", msg)
    }
}

impl DnsServer {
    pub fn new(config: ServerConfig) -> DnsServer {
        DnsServer { config }
    }

    pub fn listen(&self) {
        println!("Listening on port {}\n", self.config.port);

        let udp_socket = UdpSocket::bind(format!("127.0.0.1:{}", self.config.port))
            .expect("Failed to bind to address");

        loop {
            let mut buf = [0; 512];
            match udp_socket.recv_from(&mut buf) {
                Ok((size, source)) => {
                    println!("Received {} bytes from {}", size, source);
                    let mut request = DnsPacket::from_buf(&buf);
                    request.print_summary();

                    let mut response = self.get_response(&mut request);

                    udp_socket
                        .send_to(&response.encode(), source)
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

        let opcode = request.get_opcode();
        response.set_opcode(opcode);

        if opcode != 0 {
            response.set_rcode(ResponseCodes::NotImplemented);

            return response;
        }

        for question in request.get_questions() {
            let answers = self.get_answers(question.clone(), &mut response);

            match answers {
                Ok(answers) => {
                    response.push_question(question.clone());
                    for answer in answers {
                        response.push_answer(answer);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get answers: {}\n", e);

                    response.set_rcode(ResponseCodes::ServerFailure);

                    break;
                }
            }
        }

        return response;
    }

    fn get_answers(
        &self,
        question: Question,
        packet: &mut DnsPacket,
    ) -> Result<Vec<Answer>, ServerError> {
        if let Some(resolver_address) = &self.config.resolver {
            let mut resolver_req = DnsPacket::new_query();

            resolver_req.push_question(question);

            resolver_req.set_rd(1);

            let resolver_socket = UdpSocket::bind("0.0.0.0:0")?;

            resolver_socket
                .send_to(&resolver_req.encode(), resolver_address)
                .expect("Failed to send to resolver");

            let mut res_buf = [0u8; 512];

            resolver_socket
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();

            let (size, _) = resolver_socket.recv_from(&mut res_buf)?;

            println!("Received {} bytes from resolver", size);
            let response = DnsPacket::from_buf(&res_buf);
            response.print_summary();

            if response.get_rcode() != 0 {
                if response.get_rcode() == 3 {
                    let authorities = response.get_authorities();
                    if authorities.len() == 0 {
                        return Err(ServerError::Resolver(String::from(
                            "No answer from resolver",
                        )));
                    }
                    packet.push_authority(authorities[0].clone());
                    return Ok(vec![]);
                }
                return Err(ServerError::Resolver(format!(
                    "Resolver returned error code: {}",
                    response.get_rcode()
                )));
            }

            Ok(response.get_answers())
        } else {
            Ok(vec![Answer {
                question,
                ttl: 2200,
                data: b"\x00\x00\x00\x00".to_vec(),
            }])
        }
    }
}
