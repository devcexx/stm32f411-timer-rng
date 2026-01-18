impl rand_core::RngCore for crate::TimerRng {
    fn next_u32(&mut self) -> u32 {
        u32::from_ne_bytes([
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
        ])
    }

    fn next_u64(&mut self) -> u64 {
        u64::from_ne_bytes([
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
            self.next_byte(),
        ])
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.next_bytes(dest);
    }
}
