use std::{str::FromStr, collections::{HashSet}, cmp::max, time::Instant, fs::File};

use libspartan::{Instance, InputsAssignment, VarsAssignment, NIZKGens, NIZK};
use merlin::Transcript;
use r1cs::{GadgetBuilder, Field, num::{BigUint, Zero}, Expression, PoseidonBuilder, MdsMatrix, Element, values, MiMCBlockCipher, DaviesMeyer, MerkleDamgard, HashFunction, Gadget, WireValues, Wire, Constraint, BlockCipher};
use lazy_static::lazy_static;

mod chacha20;
mod chacha_dbg;

macro_rules! profile {
    ($tag:expr, $code:block) => {
        {
            println!("[{}] START", $tag);
            let start = Instant::now();
            let ret = $code;
            let end = Instant::now();
            println!("[{}] END: {} ms", $tag, end.duration_since(start).as_millis());
            ret
        }
    };
}

#[derive(Debug)]
struct Curve25519 {}

lazy_static! {
    static ref FIELD_ORDER: BigUint = BigUint::from_str(
        "7237005577332262213973186563042994240857116359379907606001950938285454250989"
    ).unwrap();
}

impl Field for Curve25519 {
    fn order() -> BigUint {
        FIELD_ORDER.clone()
    }
}

type F = Curve25519;

fn hash<F: Field>(
    builder: &mut GadgetBuilder<F>,
    blocks: &[Expression<F>]
) -> Expression<F> {
    let cipher = MiMCBlockCipher::default();
    let compress = DaviesMeyer::new(cipher);
    let hash = MerkleDamgard::new_defaults(compress);
    hash.hash(builder, blocks)
}

// row, col, scalar bytes
type ConstraintData = (usize, usize, [u8; 32]);

fn public_inputs_to_spartan(wire_values: &WireValues<F>) -> InputsAssignment {
    let inputs: Vec<_> = convert_wire_values(wire_values.as_map().iter()).iter()
        .map(|v| v.value)
        .collect();
    InputsAssignment::new(&inputs).unwrap()
}

fn private_inputs_to_spartan(wire_values: &WireValues<F>, public_wires: &HashSet<Wire>) -> VarsAssignment {
    let vars: Vec<_> = convert_wire_values(wire_values.as_map().iter()
        .filter(|(w, _)| !public_wires.contains(w))).iter()
        .map(|v| v.value)
        .collect();
    VarsAssignment::new(&vars).unwrap()
}

fn circuit_to_spartan(
    gadget: &Gadget<F>,
    public_wires: &[Wire],
    private_wires: &[Wire],
) -> (Instance, NIZKGens) {
    println!("map circuit");

    /* constraints */
    let constraints = gadget.constraints.iter()
        .map(convert_constraint)
        .collect();

    /* witness */
    let public_witness = convert_wires(public_wires);
    let private_witness = convert_wires(private_wires);

    let inputs = public_witness;
    let witness = private_witness;

    /* to spartan */
    println!("constraints");
    let non_zero_entries = count_non_zero(&constraints);
    let mut A: Vec<ConstraintData> = Vec::new();
    let mut B: Vec<ConstraintData> = Vec::new();
    let mut C: Vec<ConstraintData> = Vec::new();

    let mut i = 0;
    for BilinearConstraint { a, b, c } in &constraints {
        for Variable { id, value } in a {
            A.push((i, translate_id(&witness, &inputs, *id), value.clone()));
        }
        for Variable { id, value } in b {
            B.push((i, translate_id(&witness, &inputs, *id), value.clone()));
        }
        for Variable { id, value } in c {
            C.push((i, translate_id(&witness, &inputs, *id), value.clone()));
        }
        i += 1;
    }

    println!("create inst");
    let inst = Instance::new(
        constraints.len(),
        witness.len(),
        inputs.len(),
        &A,
        &B,
        &C,
    ).unwrap();

    println!("#constraints={} #variables={} #inputs={} #non_zero_entries={}",
        constraints.len(),
        witness.len(),
        inputs.len(),
        non_zero_entries,
    );

    // Create proof public params
    println!("public params (nizk)");
    let gens = NIZKGens::new(constraints.len(), witness.len(), inputs.len());

    (inst, gens)
}

fn translate_id(
    witness: &Vec<Variable>,
    inputs: &Vec<Variable>,
    id: u64,
) -> usize {
    let num_vars = witness.len();
    match witness.iter().position(|v| v.id == id) {
        Some(idx) => idx,
        None => match inputs.iter().position(|v| v.id == id) {
            Some(idx) => idx + num_vars + 1,
            None => num_vars
        }
    }
}

fn count_non_zero(constraints: &Vec<BilinearConstraint>) -> usize {
    let mut count_a = 0;
    let mut count_b = 0;
    let mut count_c = 0;
    for BilinearConstraint { a, b, c } in constraints {
        count_a += a.iter().filter(|&v| v.value.iter().any(|&x| x != 0)).count();
        count_b += b.iter().filter(|&v| v.value.iter().any(|&x| x != 0)).count();
        count_c += c.iter().filter(|&v| v.value.iter().any(|&x| x != 0)).count();
    }

    max(count_a, max(count_b, count_c))
}

struct BilinearConstraint {
    a: Vec<Variable>,
    b: Vec<Variable>,
    c: Vec<Variable>,
}

fn convert_constraint<'a, F: Field>(
    constraint: &Constraint<F>,
) -> BilinearConstraint {
    let Constraint { a, b, c } = constraint;

    BilinearConstraint {
        a: convert_expression(a),
        b: convert_expression(b),
        c: convert_expression(c),
    }
}

struct Variable {
    id: u64,
    value: [u8; 32],
}

fn element_to_bytes_le<F: Field>(n: &Element<F>) -> [u8; 32] {
    let mut buf = [0u8; 32];
    let bytes = n.to_biguint().to_bytes_le();
    buf[..bytes.len()].copy_from_slice(&bytes);
    buf
}

fn convert_expression<'a, F: Field>(
    exp: &Expression<F>,
) -> Vec<Variable> {
    exp.coefficients().iter()
        .map(|(wire, coefficient)| Variable {
            id: wire.index as u64,
            value: element_to_bytes_le(coefficient),
        })
        .collect()
}

fn convert_wire_values<'a, F: Field, I>(iter: I) -> Vec<Variable>
where
    I: Iterator<Item = (&'a Wire, &'a Element<F>)>,
{
    iter
        .map(|(wire, value)| Variable {
            id: wire.index as u64,
            value: element_to_bytes_le(value),
        })
        .collect()
}

fn convert_wires(wires: &[Wire]) -> Vec<Variable> {
    wires.iter()
        .map(|wire| Variable {
            id: wire.index as u64,
            value: [0u8; 32],
        })
        .collect()
}

fn gen_curve_key<F: Field>() -> ([u8; 32], BigUint) {
    let order = F::order();
    // Rejection sampling
    loop {
        let bytes: [u8; 32] = rand::random();
        let num = BigUint::from_bytes_le(&bytes);
        if num < order {
            return (bytes, num);
        }
    }
}

struct PrivateInputs {
    enc_key: BigUint,
    msg1: BigUint,
    msg2: BigUint,
}

#[derive(Debug)]
struct AuditOutputs {
    enc_msg1: BigUint,
    enc_msg2: BigUint,
    commit: BigUint,
    msg_hash: BigUint,
}

struct AuditGadgetEval {
    gadget: Gadget<F>,

    in_priv_enc_key: Wire,
    in_priv_orig_msg1: Wire,
    in_priv_orig_msg2: Wire,

    out_enc_msg1: Expression<F>,
    out_enc_msg2: Expression<F>,
    out_commit: Expression<F>,
    out_msg_hash: Expression<F>,
}

impl AuditGadgetEval {
    fn new() -> Self {
        let (builder, in_priv_enc_key, in_priv_orig_msg1, in_priv_orig_msg2, out_enc_msg1, out_enc_msg2, out_commit, out_msg_hash) = build_gadget_base();

        Self {
            gadget: builder.build(),
            in_priv_enc_key,
            in_priv_orig_msg1,
            in_priv_orig_msg2,
            out_enc_msg1,
            out_enc_msg2,
            out_commit,
            out_msg_hash,
        }
    }

    fn eval(&self, in_priv: &PrivateInputs) -> AuditOutputs {
        let mut values = values!(
            self.in_priv_enc_key => in_priv.enc_key.clone().into(),
            self.in_priv_orig_msg1 => in_priv.msg1.clone().into(),
            self.in_priv_orig_msg2 => in_priv.msg2.clone().into()
        );

        // Calculate outputs
        let satisfied = self.gadget.execute(&mut values);
        assert!(satisfied);

        AuditOutputs {
            enc_msg1: self.out_enc_msg1.evaluate(&values).to_biguint().clone(),
            enc_msg2: self.out_enc_msg2.evaluate(&values).to_biguint().clone(),
            commit: self.out_commit.evaluate(&values).to_biguint().clone(),
            msg_hash: self.out_msg_hash.evaluate(&values).to_biguint().clone(),
        }
    }
}

struct AuditGadgetAssert {
    pub gadget: Gadget<F>,

    in_priv_enc_key: Wire,
    in_priv_orig_msg1: Wire,
    in_priv_orig_msg2: Wire,

    in_pub_msg_hash: Wire,
    in_pub_commit: Wire,
    in_pub_enc_msg1: Wire,
    in_pub_enc_msg2: Wire,

    prover_inst: Instance,
    prover_gens: NIZKGens,
}

impl AuditGadgetAssert {
    fn new() -> Self {
        let (mut builder, in_priv_enc_key, in_priv_orig_msg1, in_priv_orig_msg2, out_enc_msg1, out_enc_msg2, out_commit, out_msg_hash) = build_gadget_base();

        let in_pub_msg_hash = builder.wire();
        let in_pub_commit = builder.wire();
        let in_pub_enc_msg1 = builder.wire();
        let in_pub_enc_msg2 = builder.wire();

        let in_pub_msg_hash_exp = Expression::from(&in_pub_msg_hash);
        let in_pub_commit_exp = Expression::from(&in_pub_commit);
        let in_pub_enc_msg1_exp = Expression::from(&in_pub_enc_msg1);
        let in_pub_enc_msg2_exp = Expression::from(&in_pub_enc_msg2);

        builder.assert_equal(&out_msg_hash, &in_pub_msg_hash_exp);
        builder.assert_equal(&out_commit, &in_pub_commit_exp);
        builder.assert_equal(&out_enc_msg1, &in_pub_enc_msg1_exp);
        builder.assert_equal(&out_enc_msg2, &in_pub_enc_msg2_exp);

        let gadget = builder.build();

        // Spartan
        let mut values = values!(
            in_pub_enc_msg1 => BigUint::zero().into(),
            in_pub_enc_msg2 => BigUint::zero().into(),
            in_pub_commit => BigUint::zero().into(),
            in_pub_msg_hash => BigUint::zero().into(),
            in_priv_enc_key => BigUint::zero().into(),
            in_priv_orig_msg1 => BigUint::zero().into(),
            in_priv_orig_msg2 => BigUint::zero().into()
        );
        // Calculate witness
        gadget.execute(&mut values);
        let witness = values;

        let mut public_wires = HashSet::new();
        public_wires.insert(Wire::ONE);
        public_wires.insert(in_pub_enc_msg1);
        public_wires.insert(in_pub_enc_msg2);
        public_wires.insert(in_pub_commit);
        public_wires.insert(in_pub_msg_hash);

        let (inst, gens) = circuit_to_spartan(
            &gadget,
            &[in_pub_msg_hash, in_pub_commit, in_pub_enc_msg1, in_pub_enc_msg2, Wire::ONE],
            &witness.as_map().iter()
                .map(|(&w, _)| w)
                .filter(|w| !public_wires.contains(w))
                .collect::<Vec<_>>(),
        );

        Self {
            gadget,

            in_priv_enc_key,
            in_priv_orig_msg1,
            in_priv_orig_msg2,

            in_pub_msg_hash,
            in_pub_commit,
            in_pub_enc_msg1,
            in_pub_enc_msg2,

            prover_inst: inst,
            prover_gens: gens,
        }
    }

    fn prove(&self, in_priv: &PrivateInputs, in_pub: &AuditOutputs) -> NIZK {
        let inputs = public_inputs_to_spartan(&values!(
            self.in_pub_enc_msg1 => in_pub.enc_msg1.clone().into(),
            self.in_pub_enc_msg2 => in_pub.enc_msg2.clone().into(),
            self.in_pub_commit => in_pub.commit.clone().into(),
            self.in_pub_msg_hash => in_pub.msg_hash.clone().into()
        ));

        let mut values = values!(
            self.in_pub_enc_msg1 => in_pub.enc_msg1.clone().into(),
            self.in_pub_enc_msg2 => in_pub.enc_msg2.clone().into(),
            self.in_pub_commit => in_pub.commit.clone().into(),
            self.in_pub_msg_hash => in_pub.msg_hash.clone().into(),
            self.in_priv_enc_key => in_priv.enc_key.clone().into(),
            self.in_priv_orig_msg1 => in_priv.msg1.clone().into(),
            self.in_priv_orig_msg2 => in_priv.msg2.clone().into()
        );

        let satisfied = self.gadget.execute(&mut values);
        assert!(satisfied);
        let witness = values;

        let mut public_wires = HashSet::new();
        public_wires.insert(Wire::ONE);
        public_wires.insert(self.in_pub_enc_msg1);
        public_wires.insert(self.in_pub_enc_msg2);
        public_wires.insert(self.in_pub_commit);
        public_wires.insert(self.in_pub_msg_hash);
        let vars = private_inputs_to_spartan(&witness, &public_wires);

        // let is_sat = self.prover_inst.is_sat(&vars, &inputs).unwrap();
        // assert!(is_sat);

        let mut prover_transcript = Transcript::new(b"audit");
        NIZK::prove(
            &self.prover_inst,
            vars,
            &inputs,
            &self.prover_gens,
            &mut prover_transcript,
        )
    }

    fn verify(&self, proof: &NIZK, in_pub: &AuditOutputs) -> bool {
        let inputs = public_inputs_to_spartan(&values!(
            self.in_pub_enc_msg1 => in_pub.enc_msg1.clone().into(),
            self.in_pub_enc_msg2 => in_pub.enc_msg2.clone().into(),
            self.in_pub_commit => in_pub.commit.clone().into(),
            self.in_pub_msg_hash => in_pub.msg_hash.clone().into()
        ));

        let mut verifier_transcript = Transcript::new(b"audit");
        proof.verify(&self.prover_inst, &inputs, &mut verifier_transcript, &self.prover_gens)
            .is_ok()
    }
}

fn build_gadget_base() -> (
    GadgetBuilder<F>,
    Wire,
    Wire,
    Wire,
    Expression<F>,
    Expression<F>,
    Expression<F>,
    Expression<F>,
) {
    let mut builder = GadgetBuilder::<F>::new();
    let cipher = MiMCBlockCipher::default();

    let in_priv_enc_key = builder.wire();
    let in_priv_orig_msg1 = builder.wire();
    let in_priv_orig_msg2 = builder.wire();

    let in_priv_enc_key_exp = Expression::from(&in_priv_enc_key);
    let in_priv_orig_msg1_exp = Expression::from(&in_priv_orig_msg1);
    let in_priv_orig_msg2_exp = Expression::from(&in_priv_orig_msg2);

    // Encrypt first. Hash moves
    let out_enc_msg1 = cipher.encrypt(&mut builder, &in_priv_enc_key_exp, &in_priv_orig_msg1_exp);
    let out_enc_msg2 = cipher.encrypt(&mut builder, &in_priv_enc_key_exp, &in_priv_orig_msg2_exp);

    let out_commit = hash(&mut builder, &[in_priv_enc_key_exp]);
    let out_msg_hash = hash(&mut builder, &[
        in_priv_orig_msg1_exp,
        in_priv_orig_msg2_exp,
    ]);

    (
        builder,
        in_priv_enc_key,
        in_priv_orig_msg1,
        in_priv_orig_msg2,
        out_enc_msg1,
        out_enc_msg2,
        out_commit,
        out_msg_hash,
    )
}

/*
nonce specified externally for circuit

pub inputs:
commitment
encrypted msg

priv inputs:
enc key
orig msg

output:
msg hash

assert:
commitment == hash(enc key)
encrypted msg = enc(orig msg, enc key)
return hash(orig msg)   // could laos be assert hash == 

----------------------------------------------------------------------

compute version:

pub inputs:
none

priv inputs:
enc key
orig msg

outputs:
commitment
enc msg
msg hash

compute:
commitment = hash(enc key)
encrypted msg = enc(orig msg, enc key)
msg hash = hash(orig msg)
 */
fn main() {
    // let mds_matrix_rows: Vec<Vec<Element<F>>> = vec![
    //     vec![
    //         BigUint::from_str_radix("28b15d6eed95eea7ebb45451308179edb7202e21a753d9cb368316b3d285219", 16).unwrap().into(),
    //         BigUint::from_str_radix("74644036d7bfabfb4e77949ca57cb10cc53cb683f406ee11d0b199074334be8", 16).unwrap().into(),
    //         BigUint::from_str_radix("9b5c144f8266c0d667a4b1bb18bd1c4ad6ca9ebbafe27d804e4964234051282", 16).unwrap().into(),
    //     ],
    //     vec![
    //         BigUint::from_str_radix("e14502eb1fdcc85376cb9d7eaa622f17e692dc175ae0508442d8598f380265b", 16).unwrap().into(),
    //         BigUint::from_str_radix("521bec0db4e14a6fffad3ca794eab19618b0aec5bac29a8305df800b5fbc430", 16).unwrap().into(),
    //         BigUint::from_str_radix("db87bc574f48be56ea4eecd0f3d79d30ad08ad5d8a9a713575ada3251ad75d1", 16).unwrap().into(),
    //     ],
    //     vec![
    //         BigUint::from_str_radix("e2845dc7c8160c536291b53080bf3f9e1a0839cbf071a3a1fc5d3cbbf073bbc", 16).unwrap().into(),
    //         BigUint::from_str_radix("0b0a7dedbd7d5b8e2678f9f1978505a90d47020173d004ef48b305ac3674660", 16).unwrap().into(),
    //         BigUint::from_str_radix("b60fdf89da602fde4467b449aa34733d5e9e545230d8b16246676c0a7af06e7", 16).unwrap().into(),
    //     ],
    // ];

    // let poseidon_n = PoseidonBuilder::new(3 /*t*/)
    //     .sbox(r1cs::PoseidonSbox::Exponentiation5)
    //     .mds_matrix(MdsMatrix::new(mds_matrix_rows))
    //     .security_bits(128)
    //     .build();

    // 256 bits (128x2)
    let msg: [u8; 32] = rand::random();
    let msg_n1 = BigUint::from_bytes_le(&msg[16..32]);
    let msg_n2 = BigUint::from_bytes_le(&msg[0..16]);
    println!("msg_n1: {}", msg_n1);
    println!("msg_n2: {}", msg_n2);

    // 252 bits
    let (enc_key, enc_key_n) = gen_curve_key::<F>();
    println!("enc_key_n: {}", enc_key_n);

    let inputs = PrivateInputs {
        enc_key: enc_key_n,
        msg1: msg_n1,
        msg2: msg_n2,
    };

    let eval_gadget = AuditGadgetEval::new();
    let outputs = profile!("eval", {
        eval_gadget.eval(&inputs)
    });
    println!("outputs: {:#?}", outputs);

    let assert_gadget = AuditGadgetAssert::new();
    for _ in 0..1000 {
        let proof = profile!("prove", {
            assert_gadget.prove(&inputs, &outputs)
        });

        let file = File::create("proof.json").unwrap();
        serde_json::to_writer(file, &proof).unwrap();

        let file = File::create("proof.bin").unwrap();
        bincode::serialize_into(&file, &proof).unwrap();
    
        let satisfied = profile!("verify", {
            assert_gadget.verify(&proof, &outputs)
        });
        assert!(satisfied);
    }
}
