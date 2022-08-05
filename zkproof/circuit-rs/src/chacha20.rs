use std::fmt::Debug;

use r1cs::{GadgetBuilder, Field, BinaryExpression, num::{BigUint, One}, BooleanExpression};

const MIX_CONST: u128 = 0x657870616e642033322d62797465206b;

type ChaChaBlock<F> = [BinaryExpression<F>; 16];

pub struct R1csChaCha20<'a, F: Field + Debug> {
    builder: &'a mut GadgetBuilder<F>,
    counter: u32,
    nonce: u128,
}

impl<'a, F: Field + Debug> R1csChaCha20<'a, F> {
    pub fn new(builder: &'a mut GadgetBuilder<F>, counter: u32, nonce: u128) -> Self {
        R1csChaCha20 {
            builder,
            counter,
            nonce,
        }
    }

    fn quarter_round(
        &mut self,
        block: &mut ChaChaBlock<F>,
        ai: usize,
        bi: usize,
        ci: usize,
        di: usize,
    ) {
        let a = &block[ai];
        let b = &block[bi];
        let c = &block[ci];
        let d = &block[di];
    
        let a = self.builder.binary_sum_ignoring_overflow(&a, &b);
        // TODO: PR to remove useless type args?
        let d_a = self.builder.bitwise_xor::<BinaryExpression<F>, BinaryExpression<F>>(&d, &a);
        let d = self.builder.bitwise_rotate_inc_significance(&d_a, 16);
    
        let c = self.builder.binary_sum_ignoring_overflow(&c, &d);
        let b_c = self.builder.bitwise_xor::<BinaryExpression<F>, BinaryExpression<F>>(&b, &c);
        let b = self.builder.bitwise_rotate_inc_significance(&b_c, 12);
    
        let a = self.builder.binary_sum_ignoring_overflow(&a, &b);
        let d_a = self.builder.bitwise_xor::<BinaryExpression<F>, BinaryExpression<F>>(&d, &a);
        let d = self.builder.bitwise_rotate_inc_significance(&d_a, 8);
    
        let c = self.builder.binary_sum_ignoring_overflow(&c, &d);
        let b_c = self.builder.bitwise_xor::<BinaryExpression<F>, BinaryExpression<F>>(&b, &c);
        let b = self.builder.bitwise_rotate_inc_significance(&b_c, 7);
    
        block[ai] = a;
        block[bi] = b;
        block[ci] = c;
        block[di] = d;
    }
    
    fn scramble_block(&mut self, block: &mut ChaChaBlock<F>) {
        // 20 rounds per block, 2 rounds per loop iteration
        for i in 0..4 {
            println!("round {}", i*2);
            // Column round (4 quarter rounds => 1 full round)
            self.quarter_round(block, 0, 4,  8, 12); // col 0
            self.quarter_round(block, 1, 5,  9, 13); // col 1
            self.quarter_round(block, 2, 6, 10, 14); // col 2
            self.quarter_round(block, 3, 7, 11, 15); // col 3
    
            // Diagonal round (4 quarter rounds => 1 full round)
            println!("round {}", i*2+1);
            self.quarter_round(block, 0, 5, 10, 15);
            self.quarter_round(block, 1, 6, 11, 12);
            self.quarter_round(block, 2, 7,  8, 13);
            self.quarter_round(block, 3, 4,  9, 14);
        }
    }

    fn process_block(&mut self, block: &mut ChaChaBlock<F>) {
        // Save original block for adding later
        let unscrambled = block.clone();
    
        // Scramble
        self.scramble_block(block);
    
        // Add original
        for i in 0..16 {
            block[i] = self.builder.binary_sum_ignoring_overflow(&block[i], &unscrambled[i]);
        }
    }

    pub fn encrypt_block(&mut self, key_exp: &BinaryExpression<F>, data_exp: &BinaryExpression<F>) -> BinaryExpression<F> {
        assert!(key_exp.len() == 256);
        assert!(data_exp.len() == 512);

        let mut block_vec = Vec::new();
        // 16 const
        block_vec.extend_from_slice(&to_binary_expr(MIX_CONST, 128).chunks(32));
        println!("len cn: {}", block_vec.len());
        // 32 key
        block_vec.extend_from_slice(&key_exp.chunks(32));
        println!("len k: {}", block_vec.len());
        // 4 counter
        block_vec.extend_from_slice(&to_binary_expr32(self.counter).chunks(32));
        println!("len c: {}", block_vec.len());
        // 12 nonce
        block_vec.extend_from_slice(&to_binary_expr(self.nonce, 96).chunks(32));
        println!("len n: {}", block_vec.len());

        let mut block: ChaChaBlock<F> = block_vec.try_into().unwrap();
        self.process_block(&mut block);
        let keystream_exp = BinaryExpression::concat(&mut block);
        println!("ln {}", keystream_exp.len());

        println!("post");
        block.iter().enumerate().for_each(|(i, b)| {
            println!("{} {}", i, b.len());
        });
        self.builder.bitwise_xor::<BinaryExpression<F>, BinaryExpression<F>>(&data_exp, &keystream_exp)
    }
}

fn to_binary_expr32<F: Field>(value: u32) -> BinaryExpression<F> {
    let bits = (0..32).map(|i| {
        let b = ((value >> i) & 1).is_one();
        BooleanExpression::from(b)
    }).collect();
    BinaryExpression { bits }
}

fn to_binary_expr<F: Field>(value: u128, bits: u8) -> BinaryExpression<F> {
    let bits = (0..bits).map(|i| {
        let b = ((value >> i) & 1).is_one();
        BooleanExpression::from(b)
    }).collect();
    BinaryExpression { bits }
}
