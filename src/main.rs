use codecrafters_dns_server::Message;
use std::net::UdpSocket;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut response = Message::from_buf(&buf[0..12]);
                response.id_from_buf(&buf);
                response.qr(true);
                response.add_question();
                response.add_answer();
                let opcode = if response.get_opcode() == 0u8 { 0 } else { 4u8 };
                println!("Found rcode: {}", opcode);
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
