#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsrsV5State {
    pub stability: f32,
    pub difficulty: f32,
}

impl Default for FsrsV5State {
    fn default() -> Self {
        Self {
            stability: 0.0,
            difficulty: 0.0,
        }
    }
}
