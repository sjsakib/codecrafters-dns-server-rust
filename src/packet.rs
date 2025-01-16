use crate::buffer_helpers::{BufferGetters, BufferSetters};
use bytes::BufMut;
use std::{net::Ipv4Addr, vec};

pub struct Parser<'b> {
    buf: &'b [u8],
    cur: usize,
    packet: DnsPacket,
}

impl<'b> Parser<'b> {
    pub fn new(buf: &[u8]) -> Parser {
        Parser {
            buf,
            cur: 0,
            packet: DnsPacket::new(),
        }
    }

    pub fn parse(&mut self) {
        self.packet.copy_head_from_slice(&self.buf[0..12]);
        self.cur = 12;

        self.packet.print_header();

        self.parse_questions();
        self.parse_answers();
    }

    pub fn get(self) -> DnsPacket {
        self.packet
    }

    fn parse_answers(&mut self) {
        let count = self.packet.get_an_count();

        for _ in 0..count {
            let question = self.parse_question();

            let ttl = self.get_u32();
            let _ = self.get_u8();
            let data = self.get_u32();

            self.packet.push_answer(Answer {
                question,
                ttl,
                data,
            });
        }
    }

    fn parse_questions(&mut self) {
        let count = self.packet.get_qd_count();

        for _ in 0..count {
            let question = self.parse_question();
            self.packet.push_question(question);
        }
    }

    fn parse_question(&mut self) -> Question {
        Question {
            labels: self.parse_labels(),
            class: self.get_u16(),
            typ: self.get_u16(),
        }
    }

    fn parse_labels(&mut self) -> Vec<String> {
        let mut labels: Vec<String> = vec![];

        let mut maybe_labels_end_position: Option<usize> = None; // TODO: smells

        while let Some((label, maybe_end_position)) = self.parse_label() {
            labels.push(label);

            if let Some(end_position) = maybe_end_position {
                maybe_labels_end_position = Some(end_position);
            }
        }

        if let Some(labels_end_position) = maybe_labels_end_position {
            self.cur = labels_end_position;
        }

        labels
    }

    fn parse_label(&mut self) -> Option<(String, Option<usize>)> {
        let len = self.get_u8();

        if len == 0 {
            return None;
        }

        if len > 63 {
            let pointer = ((len & 0b00111111) as u16 + self.get_u8() as u16) as usize;

            let cur = self.cur;

            self.cur = pointer;
            let len = self.get_u8() as usize;
            let label = self.parse_string(len);

            return Some((label, Some(cur)));
        }
        let label = self.parse_string(len as usize);

        Some((label, None))
    }

    fn parse_string(&mut self, len: usize) -> String {
        let str = String::from_utf8(self.buf[self.cur..self.cur + len].to_vec()).unwrap();

        self.cur += len;

        str
    }

    fn get_u8(&mut self) -> u8 {
        self.forward();
        self.buf[self.cur - 1]
    }

    fn get_u16(&mut self) -> u16 {
        self.cur += 2;

        self.buf.get_u16((self.cur - 2) * 8)
    }

    fn get_u32(&mut self) -> u32 {
        self.cur += 4;

        self.buf.get_u32((self.cur - 4) * 8)
    }

    fn forward(&mut self) {
        self.cur += 1;
    }
}

pub struct DnsPacket {
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
    pub fn encode(&mut self) -> Vec<u8> {
        let mut encoded = vec![];

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
    fn encode(&mut self) -> Vec<u8> {
        let mut encoded = self.question.encode();

        encoded.put_u32(self.ttl);
        encoded.put_u16(4);
        encoded.put_u32(self.data);

        encoded
    }
}

impl DnsPacket {
    pub fn new() -> DnsPacket {
        DnsPacket {
            head: [0; 12],
            questions: vec![],
            answers: vec![],
        }
    }

    pub fn new_query() -> DnsPacket {
        let mut packet = DnsPacket::new();
        packet.set_id(rand::random());

        packet
    }

    pub fn copy_head_from_slice(&mut self, buf: &[u8]) {
        self.head.copy_from_slice(&buf[0..12]);
    }

    pub fn from_buf(buf: &[u8]) -> DnsPacket {
        let mut parser = Parser::new(buf);

        parser.parse();

        parser.get()
    }

    fn print_header(&self) {
        println!("ID: {}", self.get_id());
        println!("QR: {}", self.get_qr());
        println!("OPCODE: {}", self.get_opcode());
        println!("AA: {}", self.get_aa());
        println!("TC: {}", self.get_tc());
        println!("RD: {}", self.get_rd());
        println!("RA: {}", self.get_ra());
        println!("Z: {}", self.get_z());
        println!("RCODE: {}", self.get_rcode());
        println!("QD_COUNT: {}", self.get_qd_count());
        println!("AN_COUNT: {}", self.get_an_count());
        println!("NS_COUNT: {}", self.get_ns_count());
        println!("AR_COUNT: {}", self.get_ar_count());
    }

    pub fn print(&self) {
        self.print_header();

        for q in &self.questions {
            println!("Questions: {:?}", q);
        }

        for a in &self.answers {
            println!("Answers: {:?}", a);
        }
    }

    fn get_id(&self) -> u16 {
        self.head.get_u16(0)
    }

    fn get_qr(&self) -> u8 {
        self.head.get_bit(16)
    }

    fn get_aa(&self) -> u8 {
        self.head.get_bit(17)
    }

    fn get_tc(&self) -> u8 {
        self.head.get_bit(18)
    }

    fn get_rd(&self) -> u8 {
        self.head.get_bit(24)
    }

    fn get_ra(&self) -> u8 {
        self.head.get_bit(25)
    }

    fn get_z(&self) -> u8 {
        self.head.get_bit(26)
    }

    pub fn get_opcode(&self) -> u8 {
        self.head.get_u4(17)
    }

    pub fn get_rcode(&self) -> u8 {
        self.head.get_u4(28)
    }

    fn get_qd_count(&self) -> u16 {
        self.head.get_u16(32)
    }

    pub fn get_an_count(&self) -> u16 {
        self.head.get_u16(48)
    }

    fn get_ns_count(&self) -> u16 {
        self.head.get_u16(64)
    }

    fn get_ar_count(&self) -> u16 {
        self.head.get_u16(80)
    }

    fn set_id(&mut self, id: u32) {
        self.head.put_u16(0, id as u16);
    }

    pub fn set_qr(&mut self, f: u8) {
        self.head.put_bit(16, f);
    }

    pub fn set_ar_count(&mut self, count: u16) {
        self.head.put_u16(80, count);
    }

    pub fn set_rd(&mut self, f: u8) {
        self.head.put_bit(24, f)
    }

    pub fn set_rcode(&mut self, rcode: u8) {
        self.head.put_u4(28, rcode);
    }

    fn set_qd_count(&mut self, count: u16) {
        self.head.put_u16(32, count);
    }

    fn set_a_count(&mut self, count: u16) {
        self.head.put_u16(48, count);
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

    pub fn push_question(&mut self, question: Question) {
        self.questions.push(question);
        self.set_qd_count(self.questions.len() as u16);
    }

    pub fn get_questions(&self) -> Vec<Question> {
        self.questions.clone()
    }

    pub fn get_answers(&self) -> Vec<Answer> {
        self.answers.clone()
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
}
