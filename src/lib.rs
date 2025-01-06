use std::net::Ipv4Addr;
use bytes::{BufMut, BytesMut};

fn split_u16(n: u16) -> (u8, u8) {
    ((n >> 8) as u8, (n & 0xFF) as u8)
}

pub struct Message {
    head: [u8; 12],
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

#[derive(Debug, Clone)]
pub struct Question {
    labels: Vec<String>,
    class: u16,
    typ: u16,
}
impl Question {
    pub fn encode(&mut self) -> BytesMut {
        let mut encoded = BytesMut::new();

        for label in &mut self.labels {
            encoded.put_u8(label.len() as u8);
            let bytes: &[u8] = label.as_bytes();
            encoded.put_slice(bytes);
        }

        encoded.put_u8(0u8);
        encoded.put_u16(self.typ);
        encoded.put_u16(self.class);

        encoded
    }
}

struct Answer {
    question: Question,
    ttl: u32,
    data: u32,
}

impl Answer {
    fn encode(&mut self) -> BytesMut {
        let mut encoded = self.question.encode();

        encoded.put_u32(self.ttl);
        encoded.put_u16(4);
        encoded.put_u32(self.data);

        encoded
    }
}

impl Message {
    pub fn new() -> Message {
        Message {
            head: [0; 12],
            questions: vec![],
            answers: vec![],
        }
    }

    pub fn from_buf(buf: &[u8]) -> Message {
        let mut head = [0; 12];

        head.copy_from_slice(&buf[0..12]);

        let mut message = Message {
            head,
            questions: vec![],
            answers: vec![],
        };

        message.parse_questions(&buf[12..]);

        println!("Questions: {:?}", message.questions);

        message
    }

    fn put_bit(&mut self, offset: usize, f: bool) {
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        if f {
            self.head[byte_idx] |= 1u8 << (7 - bit_idx);
        } else {
            self.head[byte_idx] &= !(1u8 << (7 - bit_idx));
        }
    }

    fn put_u4(&mut self, offset: usize, n: u8) {
        let byte_idx = offset / 8;
        let bit_offset = offset % 8;

        self.head[byte_idx] &= !(0b00001111 << (4 - bit_offset));

        self.head[byte_idx] |= n << (4 - bit_offset);
    }

    fn get_u4(&self, offset: usize) -> u8 {
        let byte_idx = offset / 8;
        let bit_offset = offset % 8;

        (self.head[byte_idx] & (0b00001111 << (4 - bit_offset))) >> (4 - bit_offset)
    }

    /// Assumes no fractional byte is needed
    fn get_u16(&self, offset: usize) -> u16 {
        let byte_offset = offset / 8;
        let mut x = 0u16;
        x |= self.head[byte_offset + 1] as u16;
        x |= (self.head[byte_offset] as u16) << 8;

        x
    }

    pub fn get_bytes(&mut self) -> Vec<u8> {
        let mut res = vec![];
        res.append(&mut self.head.to_vec());

        for q in &mut self.questions {
            res.append(&mut q.encode().to_vec());
        }

        for q in &mut self.answers {
            res.append(&mut q.encode().to_vec());
        }

        res
    }
    pub fn id_from_buf(&mut self, buf: &[u8]) {
        self.head[0] = buf[0];
        self.head[1] = buf[1];
    }

    pub fn qr(&mut self, f: bool) {
        self.put_bit(16, f);
    }

    pub fn set_rcode(&mut self, rcode: u8) {
        self.put_u4(28, rcode);
    }

    pub fn get_opcode(&self) -> u8 {
        self.get_u4(17)
    }

    pub fn get_rcode(&self) -> u8 {
        self.get_u4(28)
    }

    // fn set_qd_count(&mut self, count: u16) {
    //     let (high, low) = split_u16(count);
    //     self.head[4] = high;
    //     self.head[5] = low;
    // }

    fn get_qd_count(&self) -> u16 {
        self.get_u16(32)
    }

    fn set_a_count(&mut self, count: u16) {
        let (high, low) = split_u16(count);
        self.head[6] = high;
        self.head[7] = low;
    }

    pub fn get_a_count(&self) -> u16 {
        self.get_u16(32)
    }

    pub fn parse_questions(&mut self, buf: &[u8]) {
        let q_count = self.get_qd_count();

        let mut i = 0;
        while i < buf.len() && (self.questions.len() as u16) < q_count {
            let mut labels: Vec<String> = vec![];
            let mut label_len = buf[i] as usize;

            while label_len > 0 {
                i += 1;
                let label = String::from_utf8(buf[i..i + label_len].to_vec()).unwrap();
                labels.push(label);

                i += label_len;
                label_len = buf[i] as usize;
            }
            i += 1;
            self.questions.push(Question {
                labels,
                typ: ((buf[i] as u16) << 8) + buf[i + 1] as u16,
                class: ((buf[i + 2] as u16) << 8) + buf[i + 3] as u16,
            });

            i += 4;
        }
    }
    pub fn add_answer(&mut self, question: Question, ttl: u32, data: Ipv4Addr) {
        let answer = Answer {
            question,
            ttl,
            data: data.to_bits(),
        };

        self.answers.push(answer);
        self.set_a_count(self.answers.len() as u16);
    }

    pub fn get_questions(&self) -> Vec<Question> {
        self.questions.clone()
    }
}
