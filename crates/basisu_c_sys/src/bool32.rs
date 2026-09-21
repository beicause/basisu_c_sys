impl Bool32 {
    /// Return `true` when the C `wasm_bool_t` is non-zero.
    pub fn is_ok(&self) -> bool {
        self.0 != 0
    }
    /// Return `true` when the C `wasm_bool_t` is zero.
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
