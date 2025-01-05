use bytes::{BufMut, BytesMut};

fn split_u16(n: u16) -> (u8, u8) {
    ((n >> 8) as u8, (n & 0xFF) as u8)
}

pub struct Message {
    head: [u8; 12],
    questions: Vec<BytesMut>,
    answers: Vec<BytesMut>,
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

        head.copy_from_slice(buf);

        Message {
            head,
            questions: vec![],
            answers: vec![],
        }
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
        let bit_offset = (offset % 8);

        self.head[byte_idx] &= !(0b00001111 << (4 - bit_offset));

        self.head[byte_idx] |= n << (4 - bit_offset);
    }

    fn get_u4(&self, offset: usize) -> u8 {
        let byte_idx = offset / 8;
        let bit_offset = (offset % 8);

        (self.head[byte_idx] & (0b00001111 << (4 - bit_offset))) >> (4 - bit_offset)
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut res = vec![];
        res.append(&mut self.head.to_vec());

        for q in &self.questions {
            res.append(&mut q.to_vec());
        }

        for q in &self.answers {
            res.append(&mut q.to_vec());
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

    fn set_a_count(&mut self, count: u16) {
        let (high, low) = split_u16(count);
        self.head[6] = high;
        self.head[7] = low;
    }

    fn set_opcode(&mut self, high: u8, low: u8) {
        self.head[2] &= 0x80; // clear last 7 bits
        self.head[2] |= high >> 1;
        self.head[3] &= 1;
    }

    pub fn add_question(&mut self) {
        let mut question = BytesMut::new();
        question.put(&b"\x0ccodecrafters\x02io"[..]);
        question.put_u8(0u8);
        question.put_u16(0x1);
        question.put_u16(0x1);

        self.questions.push(question);
        self.set_qd_count(self.questions.len() as u16);
    }

    pub fn add_answer(&mut self) {
        let mut answer = BytesMut::new();
        answer.put(&b"\x0ccodecrafters\x02io"[..]);
        answer.put_u8(0u8); // null byte
        answer.put_u16(0x1); // type
        answer.put_u16(0x1); // class

        answer.put_u32(100); // TTL
        answer.put_u16(4); // RDATA length

        answer.put_u8(76);
        answer.put_u8(76);
        answer.put_u8(21);
        answer.put_u8(21);

        self.answers.push(answer);
        self.set_a_count(self.answers.len() as u16);
    }
}
