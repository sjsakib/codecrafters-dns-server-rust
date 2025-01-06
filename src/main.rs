use codecrafters_dns_server::Message;
use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut response = Message::from_buf(&buf);
                response.qr(true);
                let opcode = if response.get_opcode() == 0u8 { 0 } else { 4u8 };
                response.set_rcode(opcode);

                println!("Number of questions: {}", response.get_a_count());

                for question in response.get_questions() {
                    response.add_answer(question, 2200, Ipv4Addr::new(8, 8, 8, 8))
                }


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
