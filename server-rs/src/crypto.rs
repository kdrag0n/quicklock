pub fn hash(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}
