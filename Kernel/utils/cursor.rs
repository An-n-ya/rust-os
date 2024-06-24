use alloc::vec::Vec;

pub struct Cursor {
    data: Vec<u8>,
    pos: usize,
}

impl Cursor {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    pub fn set_position(&mut self, position: usize) {
        self.pos = position;
    }
    pub fn is_empty(&self) -> bool {
        self.pos == self.data.len()
    }
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            let res = self.data[self.pos];
            self.pos += 1;
            Some(res)
        }
    }
    pub fn read(&mut self, buf: &mut [u8]) {
        let cnt = buf.len().min(self.data.len() - self.pos);
        for i in 0..cnt {
            buf[i] = self.data[self.pos + i];
        }
        self.pos += cnt;
    }
}
