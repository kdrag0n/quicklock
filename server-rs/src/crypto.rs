pub fn hash(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

pub fn hash_id_short(data: &[u8]) -> [u8; 12] {
    hash(data)[..12].try_into().unwrap()
}
