use bytes::{BufMut, BytesMut};
use std::net::Ipv4Addr;

fn split_u16(n: u16) -> (u8, u8) {
    ((n >> 8) as u8, (n & 0xFF) as u8)
}

fn parse_label(buf: &[u8], offset: usize) -> (String, usize, bool) {
    let label_len = buf[offset] as usize;

    println!("Label len: {}", label_len);

    if label_len == 0 {
        (String::from(""), offset + 1, false)
    } else if label_len > 63 {
        // 00000011000000
        // 11111100111111
        let mut off = ((((label_len as u16) & 0b1111111100111111u16) << 8) as usize)
            + (buf[offset + 1] as usize)
            - 12usize;

        println!("Codded offset: {}, bytes: {:?}", off, buf);

        let (label, end, __) = parse_label(buf, off);

        (label, end, true)
    } else {
        let label = String::from_utf8(buf[offset + 1..offset + label_len + 1].to_vec()).unwrap();

        (label, offset + label_len + 1, false)
    }
}

pub struct Message {
    pub head: [u8; 12],
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

#[derive(Clone, Debug)]
pub struct Answer {
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

trait Buf {
    fn get_u16(self, offset: usize) -> u16;
    fn get_u4(self, offset: usize) -> u8;
    fn get_u32(self, offset: usize) -> u32;
}

impl Buf for &[u8] {
    fn get_u16(self, offset: usize) -> u16 {
        let byte_offset = offset / 8;
        let mut x = 0u16;
        x |= self[byte_offset + 1] as u16;
        x |= (self[byte_offset] as u16) << 8;

        x
    }

    fn get_u4(self, offset: usize) -> u8 {
        let byte_idx = offset / 8;
        let bit_offset = offset % 8;

        (self[byte_idx] & (0b00001111 << (4 - bit_offset))) >> (4 - bit_offset)
    }

    fn get_u32(self, offset: usize) -> u32 {
        let byte_offset = offset / 8;
        let mut x = 0u32;
        x |= self[byte_offset + 3] as u32;
        x |= (self[byte_offset + 2] as u32) << 8;
        x |= (self[byte_offset + 1] as u32) << 16;
        x |= (self[byte_offset] as u32) << 24;

        x
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

    pub fn from_header_buf(buf: &[u8]) -> Message {
        let mut head = [0u8; 12];

        head.copy_from_slice(buf);

        Message {
            head,
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

        let i = message.parse_section(&buf[12..], true);

        println!("Questions: {:?}", message.questions);

        // println!("Parsing answers from offset: {}", i);
        message.parse_section(&buf[12 + i..], false);

        println!("Answers: {:?}", message.answers);

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
        self.head.get_u4(offset)
    }

    /// Assumes no fractional byte is needed
    fn get_u16(&self, offset: usize) -> u16 {
        self.head.get_u16(offset)
    }

    fn get_u32(&self, offset: usize) -> u32 {
        self.head.get_u32(offset)
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

    fn set_qd_count(&mut self, count: u16) {
        let (high, low) = split_u16(count);
        self.head[4] = high;
        self.head[5] = low;
    }

    fn get_qd_count(&self) -> u16 {
        self.get_u16(32)
    }

    fn set_a_count(&mut self, count: u16) {
        let (high, low) = split_u16(count);
        self.head[6] = high;
        self.head[7] = low;
    }

    pub fn get_a_count(&self) -> u16 {
        self.get_u16(48)
    }

    pub fn parse_section(&mut self, buf: &[u8], is_question: bool) -> usize {
        // println!("bytes: {:?}", buf);
        let total_count = if is_question {
            self.get_qd_count()
        } else {
            self.get_a_count()
        };

        // println!("Total count: {}", total_count);

        let mut count = 0;

        let mut i = 0;
        // while i < buf.len() && (self.questions.len() as u16) < count {
        while i < buf.len() && count < total_count {
            let mut labels: Vec<String> = vec![];
            loop {
                let (label, end, was_compressed) = parse_label(buf, i);

                if was_compressed {
                    labels.push(label);
                    let mut ii = end;
                    loop {
                        let (l, e, _) = parse_label(buf, ii);

                        if e == ii + 1 {
                            break;
                        }
                        labels.push(l);
                        ii = e;

                    }
                    i += 1;
                    break;

                }

                if end == i + 1  {
                    // if was_compressed {
                    //     labels.push(label);
                    //     i = end - 1;
                    // }
                    break;
                }
                labels.push(label);
                i = end;
            }
            i += 1;

            println!("Labels: {:?} {}", labels, i);

            let question = Question {
                labels,
                typ: buf.get_u16(i * 8),
                class: buf.get_u16((i + 2) * 8),
            };
            i += 4;
            if is_question {
                self.questions.push(question);
            } else {
                self.answers.push(Answer {
                    question,
                    ttl: buf.get_u32(i * 8),
                    data: buf.get_u32((i + 6) * 8),
                });

                i += 10;
            }
            count += 1;
        }

        i
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

    pub fn push_answer(&mut self, answer: Answer) {
        self.answers.push(answer);
        self.set_a_count(self.answers.len() as u16);
    }

    pub fn push_question(&mut self, question: &Question) {
        self.questions.push(question.clone());
        self.set_qd_count(self.questions.len() as u16);
    }

    pub fn get_questions(&self) -> Vec<Question> {
        self.questions.clone()
    }

    pub fn get_answers(&self) -> Vec<Answer> {
        self.answers.clone()
    }
}
