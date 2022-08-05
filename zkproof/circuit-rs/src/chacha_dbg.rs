const MIX_CONST: &[u8; 16] = b"expand 32-byte k";

fn quarter_round(block: &mut [u8; 64], ai: usize, bi: usize, ci: usize, di: usize) {
    let mut a = u32::from_le_bytes(block[(ai*4)..(ai*4)+4].try_into().unwrap());
    let mut b = u32::from_le_bytes(block[(bi*4)..(bi*4)+4].try_into().unwrap());
    let mut c = u32::from_le_bytes(block[(ci*4)..(ci*4)+4].try_into().unwrap());
    let mut d = u32::from_le_bytes(block[(di*4)..(di*4)+4].try_into().unwrap());

    a = a.wrapping_add(b);  d = (d ^ a).rotate_left(16);
    c = c.wrapping_add(d);  b = (b ^ c).rotate_left(12);
    a = a.wrapping_add(b);  d = (d ^ a).rotate_left(8);
    c = c.wrapping_add(d);  b = (b ^ c).rotate_left(7);
    // a = b;  d = (d ^ a).rotate_left(16);
    // c = d;  b = (b ^ c).rotate_left(12);
    // a = b;  d = (d ^ a).rotate_left(8);
    // c = d;  b = (b ^ c).rotate_left(7);

    block[(ai*4)..(ai*4)+4].copy_from_slice(&a.to_le_bytes());
    block[(bi*4)..(bi*4)+4].copy_from_slice(&b.to_le_bytes());
    block[(ci*4)..(ci*4)+4].copy_from_slice(&c.to_le_bytes());
    block[(di*4)..(di*4)+4].copy_from_slice(&d.to_le_bytes());
}

fn scramble_block(block: &mut [u8; 64]) {
    for _ in 0..4 {
        println!("blk {:?}", block);
        // Column round (4 quarter rounds => 1 full round)
        quarter_round(block, 0, 4,  8, 12); // col 0
        quarter_round(block, 1, 5,  9, 13); // col 1
        quarter_round(block, 2, 6, 10, 14); // col 2
        quarter_round(block, 3, 7, 11, 15); // col 3

        // Diagonal round (4 quarter rounds => 1 full round)
        quarter_round(block, 0, 5, 10, 15);
        quarter_round(block, 1, 6, 11, 12);
        quarter_round(block, 2, 7,  8, 13);
        quarter_round(block, 3, 4,  9, 14);
    }
}

fn process_block(block: &mut [u8; 64]) {
    // Save orig
    let unscrambled = block.clone();

    // Scramble
    scramble_block(block);

    // Add orig
    for i in 0..16 {
        block[i] += unscrambled[i];
    }
}

pub fn encrypt_block(key: &[u8; 32], data: &mut [u8; 64], counter: u32, nonce: &[u8; 12]) {
    let mut block = [0u8; 64];
    block[0..16].copy_from_slice(MIX_CONST);
    block[16..48].copy_from_slice(key);
    block[48..52].copy_from_slice(&counter.to_le_bytes());
    block[52..64].copy_from_slice(nonce);

    println!("blk: {:?}", block);
    process_block(&mut block);
    println!("blk: {:?}", block);

    // Apply keystream to data
    for i in 0..64 {
        data[i] ^= block[i];
    }
}
