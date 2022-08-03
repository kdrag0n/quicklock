use std::{str::FromStr, collections::{HashSet, btree_map::Iter}, cmp::max, time::Instant, fs::File};

use curve25519_dalek::scalar::Scalar;
use dalek_ff_group::field::FieldElement;
use ff::PrimeField;
use libspartan::{Instance, InputsAssignment, VarsAssignment, SNARKGens, SNARK, NIZKGens, NIZK};
use merlin::Transcript;
use neptune::circuit2::poseidon_hash_num;
use r1cs::{GadgetBuilder, Field, num::{BigUint, Num, Zero, Integer}, Expression, PoseidonBuilder, MdsMatrix, Element, values, MiMCBlockCipher, DaviesMeyer, MerkleDamgard, HashFunction, binary_unsigned_values, Gadget, WireValues, Wire, Constraint};
// use r1cs_zkinterface::{write_circuit_and_r1cs, write_circuit_and_witness};
// use spzk::R1csReader;

macro_rules! profile {
    ($tag:expr, $code:block) => {
        {
            println!("[{}] START", $tag);
            let __profile_start = Instant::now();
            let ret = $code;
            let __profile_end = Instant::now();
            println!("[{}] END: {} ms", $tag, __profile_end.duration_since(__profile_start).as_millis());
            ret
        }
    };
}

struct Curve25519 {}

impl Field for Curve25519 {
    fn order() -> BigUint {
        BigUint::from_str(
            "7237005577332262213973186563042994240857116359379907606001950938285454250989"
        ).unwrap()
    }
}

type F = Curve25519;

// #[derive(PrimeField)]
// #[PrimeFieldModulus = "7237005577332262213973186563042994240857116359379907606001950938285454250989"]
// #[PrimeFieldGenerator = "1"]
// #[PrimeFieldReprEndianness = "little"]
// struct Fp([u64; 4]);

fn hash<F: Field>(
    builder: &mut GadgetBuilder<F>,
    blocks: &[Expression<F>]
) -> Expression<F> {
    let cipher = MiMCBlockCipher::default();
    let compress = DaviesMeyer::new(cipher);
    let hash = MerkleDamgard::new_defaults(compress);
    hash.hash(builder, blocks)
}

struct CircuitData {
    r1cs: Vec<u8>,
    witness: Vec<u8>,
}

// fn write_zkinterface(
//     gadget: &Gadget<Curve25519>,
//     wire_values: &WireValues<Curve25519>,
// ) -> CircuitData {
//     println!("Write zkinterface data");

//     let mut public_wires = HashSet::new();
//     public_wires.insert(Wire::ONE);
//     public_Wires.extend(wire_values.dependencies());

//     let mut r1cs_buf = Vec::new();
//     write_circuit_and_r1cs(gadget, &public_wires, &mut r1cs_buf);

//     let mut witness_buf = Vec::new();
//     write_circuit_and_witness(gadget, wire_values, &public_wires, &mut witness_buf);

//     CircuitData {
//         r1cs: r1cs_buf,
//         witness: witness_buf,
//     }
// }

// row, col, scalar bytes
type ConstraintData = (usize, usize, [u8; 32]);

fn create_r1cs_instance(
    gadget: &Gadget<Curve25519>,
    wire_values: &WireValues<Curve25519>,
) {
    // parameters of the R1CS instance
    let num_cons = 4;
    let num_vars = 4;
    let num_inputs = 1;
    let num_non_zero_entries = 8;

    // We will encode the above constraints into three matrices, where
    // the coefficients in the matrix are in the little-endian byte order
    let mut A: Vec<ConstraintData> = Vec::new();
    let mut B: Vec<ConstraintData> = Vec::new();
    let mut C: Vec<ConstraintData> = Vec::new();

    let inst = Instance::new(num_cons, num_vars, num_inputs, &A, &B, &C).unwrap();
}

fn export_mir_r1cs(
    gadget: &Gadget<Curve25519>,
    wire_values: &WireValues<Curve25519>,
    public_wires: &HashSet<Wire>,
) {
    println!("map circuit");

    /* circuit */
    let wires: Vec<_> = gadget.constraints.iter()
        .flat_map(|c| [c.a.dependencies(), c.b.dependencies(), c.c.dependencies()])
        .flatten()
        .collect();

    let variable_ids: Vec<_> = public_wires.iter().map(|w| w.index as u64).collect();

    /* constraints */
    let constraints = gadget.constraints.iter()
        .map(convert_constraint)
        .collect();

    /* witness */
    let public_witness = convert_wire_values(wire_values.as_map().iter()
        .filter(|(w, _)| public_wires.contains(w)));
    let private_witness = convert_wire_values(wire_values.as_map().iter()
        .filter(|(w, _)| !public_wires.contains(w)));

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

    println!("vars");
    let vars: Vec<_> = witness.iter().map(|v| v.value).collect();
    let vars_assignment = VarsAssignment::new(&vars).unwrap();

    println!("inputs");
    let inputs: Vec<_> = inputs.iter().map(|v| v.value).collect();
    let inputs_assignment = InputsAssignment::new(&inputs).unwrap();

    // Check if instance is satisfiable
    println!("is sat");
    let res = inst.is_sat(&vars_assignment, &inputs_assignment).unwrap();
    println!("Instance is satisfiable: {}", res);

    // Create proof public params
    println!("public params (nizk)");
    let gens = NIZKGens::new(constraints.len(), witness.len(), inputs.len());

    for i in 0..1000 {
        // Produce a proof of satisfiability
        let proof = profile!("prove", {
            let mut prover_transcript = Transcript::new(b"snark_example");
            NIZK::prove(
                &inst,
                vars_assignment.clone(),
                &inputs_assignment,
                &gens,
                &mut prover_transcript,
            )
        });
        
        // let file = File::create("proof.json").unwrap();
        // serde_json::to_writer(file, &proof).unwrap();

        // Verify
        profile!("verify", {
            let mut verifier_transcript = Transcript::new(b"snark_example");
            proof.verify(&inst, &inputs_assignment, &mut verifier_transcript, &gens).unwrap();
        });
        println!("proof ok");
    }
}

fn translate_id(
    witness: &Vec<Variable>,
    inputs: &Vec<Variable>,
    id: u64,
) -> usize {
    let num_vars = witness.len();
    match witness.iter().position(|v| v.id == id) {
        Some(idx) => return idx,
        None => match inputs.iter().position(|v| v.id == id) {
            Some(idx) => return idx + num_vars + 1,
            None => return num_vars
        }
    }
}

fn count_non_zero(constraints: &Vec<BilinearConstraint>) -> usize {
    let mut count_a = 0;
    let mut count_b = 0;
    let mut count_c = 0;
    for BilinearConstraint { a, b, c } in constraints {
        count_a += a.iter().filter(|&v| v.value.iter().any(|x| *x != 0)).count();
        count_b += b.iter().filter(|&v| v.value.iter().any(|x| *x != 0)).count();
        count_c += c.iter().filter(|&v| v.value.iter().any(|x| *x != 0)).count();
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

/*
pub inputs:
commitment
encrypted msg

priv inputs:
enc key
enc commit nonce
orig msg

output:
msg hash

assert:
commitment == hash(enc key + enc commit nonce)
encrypted msg = enc(orig msg, enc key)
return hash(orig msg)   // could laos be assert hash == 
 */
fn main() {
    let mds_matrix_rows: Vec<Vec<Element<F>>> = vec![
        vec![
            BigUint::from_str_radix("28b15d6eed95eea7ebb45451308179edb7202e21a753d9cb368316b3d285219", 16).unwrap().into(),
            BigUint::from_str_radix("74644036d7bfabfb4e77949ca57cb10cc53cb683f406ee11d0b199074334be8", 16).unwrap().into(),
            BigUint::from_str_radix("9b5c144f8266c0d667a4b1bb18bd1c4ad6ca9ebbafe27d804e4964234051282", 16).unwrap().into(),
        ],
        vec![
            BigUint::from_str_radix("e14502eb1fdcc85376cb9d7eaa622f17e692dc175ae0508442d8598f380265b", 16).unwrap().into(),
            BigUint::from_str_radix("521bec0db4e14a6fffad3ca794eab19618b0aec5bac29a8305df800b5fbc430", 16).unwrap().into(),
            BigUint::from_str_radix("db87bc574f48be56ea4eecd0f3d79d30ad08ad5d8a9a713575ada3251ad75d1", 16).unwrap().into(),
        ],
        vec![
            BigUint::from_str_radix("e2845dc7c8160c536291b53080bf3f9e1a0839cbf071a3a1fc5d3cbbf073bbc", 16).unwrap().into(),
            BigUint::from_str_radix("0b0a7dedbd7d5b8e2678f9f1978505a90d47020173d004ef48b305ac3674660", 16).unwrap().into(),
            BigUint::from_str_radix("b60fdf89da602fde4467b449aa34733d5e9e545230d8b16246676c0a7af06e7", 16).unwrap().into(),
        ],
    ];

    let mut builder = GadgetBuilder::<F>::new();
    let poseidon_n = PoseidonBuilder::new(3 /*t*/)
        .sbox(r1cs::PoseidonSbox::Exponentiation5)
        .mds_matrix(MdsMatrix::new(mds_matrix_rows))
        .security_bits(128)
        .build();

    println!("make r1cs");
    // 32-byte, 256-bit input data.
    // Assert hash equals

    // let in_msg_hash = builder.wire();
    let in_orig_msg = builder.wire();

    // let msg_hash_exp = Expression::from(in_msg_hash);
    let orig_msg_exp = Expression::from(in_orig_msg);

    // expect_msg_hash = hash(orig_msg)
    let expect_msg_hash = hash(&mut builder, &[orig_msg_exp]);

    // assert(msg_hash == expect_msg_hash)
    // builder.assert_equal(&msg_hash_exp, &expect_msg_hash);

    // // Avoid leaking info
    // // output = expect_msg_hash * 0 = 0
    // let out_zero = builder.product(&expect_msg_hash, &Expression::zero());

    let gadget = builder.build();

    println!("exec");
    // 252-bit message for now
    // msg mod P
    // let msg: [u8; 32] = rand::random();
    let msg = [0u8; 32];
    let msg_n = BigUint::from_bytes_le(&msg);
    let msg_clamped = msg_n.mod_floor(&Curve25519::order());

    let mut values = values!(in_orig_msg => msg_clamped.into());
    let satisfied = gadget.execute(&mut values);
    assert!(satisfied);

    // Get the hash
    let out_hash = profile!("exec circuit", {
        expect_msg_hash.evaluate(&values)
    });
    println!("out_hash {:?}", out_hash.to_biguint());


    let mut public_wires = HashSet::new();
    public_wires.insert(Wire::ONE);
    public_wires.extend(expect_msg_hash.dependencies());
    // let circuit_data = write_zkinterface(&gadget, &values);

    println!("\ntry spartan");
    export_mir_r1cs(&gadget, &values, &public_wires);
}
