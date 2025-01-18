pub trait BufferGetters {
    fn get_bit(self, offset: usize) -> u8;
    fn get_u4(self, offset: usize) -> u8;
    fn get_u16(self, offset: usize) -> u16;
    fn get_u32(self, offset: usize) -> u32;
}

/// Assumes that byte multiple fractional bytes are never needed to get a value
impl BufferGetters for &[u8] {
    fn get_bit(self, offset: usize) -> u8 {
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        (self[byte_idx] << bit_idx) >> 7
    }

    fn get_u4(self, offset: usize) -> u8 {
        let byte_idx = offset / 8;
        let bit_offset = offset % 8;

        (self[byte_idx] & (0b00001111 << (4 - bit_offset))) >> (4 - bit_offset)
    }

    fn get_u16(self, offset: usize) -> u16 {
        let byte_offset = offset / 8;

        (self[byte_offset + 1] as u16) + ((self[byte_offset] as u16) << 8)
    }

    fn get_u32(self, offset: usize) -> u32 {
        let byte_offset = offset / 8;

        ((self[byte_offset] as u32) << 24)
            + ((self[byte_offset + 1] as u32) << 16)
            + ((self[byte_offset + 2] as u32) << 8)
            + (self[byte_offset + 3] as u32)
    }
}

pub trait BufferSetters {
    fn put_bit(self, offset: usize, bit: u8);
    fn put_u4(self, offset: usize, value: u8);
    fn put_u16(self, offset: usize, value: u16);
}

impl BufferSetters for &mut [u8] {
    fn put_bit(self, offset: usize, bit: u8) {
        let byte_idx = offset / 8;
        let bit_idx = offset % 8;

        self[byte_idx] = self[byte_idx] & !(1 << (7 - bit_idx)) | (bit << (7 - bit_idx));
    }

    fn put_u4(self, offset: usize, value: u8) {
        let byte_idx = offset / 8;
        let bit_offset = offset % 8;

        self[byte_idx] =
            self[byte_idx] & !(0x0F << (4 - bit_offset)) | (value << (4 - bit_offset));
    }

    fn put_u16(self, offset: usize, value: u16) {
        let byte_offset = offset / 8;
        self[byte_offset] = (value >> 8) as u8;
        self[byte_offset + 1] = (value & 0xFF) as u8;
    }
}
