impl Bool32 {
    pub fn is_ok(&self) -> bool {
        self.0 != 0
    }
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
