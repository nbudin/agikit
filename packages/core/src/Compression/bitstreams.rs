pub trait ReadBitstream {
    fn bit_offset(&self) -> usize;
    fn read_code(&mut self, bit_length: usize) -> Result<u32, std::io::Error>;
    fn seek_bits(&mut self, bits: isize) -> Result<(), std::io::Error>;
    fn done(&self) -> bool;

    fn byte_offset(&self) -> usize {
        self.bit_offset() / 8
    }

    fn peek_code(&mut self, bit_length: usize) -> Result<u32, std::io::Error> {
        let code = self.read_code(bit_length)?;
        self.seek_bits(-(bit_length as isize))?;
        Ok(code)
    }
}

pub trait WriteBitstream {
    fn current_byte_offset(&self) -> usize;
    fn current_byte(&self) -> u8;
    fn get_data(&self) -> &[u8];
    fn write_code(&mut self, code: u32, bit_length: usize);
    fn flush_current_byte(&mut self);

    fn finish(&mut self) -> Vec<u8> {
        if self.current_byte_offset() > 0 {
            self.flush_current_byte();
        }

        self.get_data().to_vec()
    }
}
