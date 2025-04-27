// Copyright (c) 2023 Espresso Systems (espressosys.com)
// This file is part of the HyperPlonk library.

// You should have received a copy of the MIT License
// along with the HyperPlonk library. If not, see <https://mit-license.org/>.

use std::{fs::File, io, time::Instant};

use ark_bls12_381::{Bls12_381, Fr};
use ark_ec::pairing::Pairing;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Write};
use ark_std::rand::prelude::StdRng;
use ark_std::{test_rng, Zero};
use ark_bn254::Bn254;
use hyperplonk::{
    prelude::{CustomizedGates, HyperPlonkErrors, MockCircuit},
    HyperPlonkSNARK,
};
use subroutines::{pcs::{
    prelude::{MultilinearKzgPCS, MultilinearUniversalParams},
    PolynomialCommitmentScheme,
}, poly_iop::PolyIOP, MercuryPCS, SumCheck};
use subroutines::pcs::Samaritan::SamaritanPCS;

const SUPPORTED_SIZE: usize = 20;
const MIN_NUM_VARS: usize = 8;
const MAX_NUM_VARS: usize = 20;
const MIN_CUSTOM_DEGREE: usize = 1;
const MAX_CUSTOM_DEGREE: usize = 32;
const HIGH_DEGREE_TEST_NV: usize = 15;

fn main() -> Result<(), HyperPlonkErrors> {
    let thread = rayon::current_num_threads();
    println!("start benchmark with #{} threads", thread);
    let mut rng = test_rng();

    test_hyperplonk_e2e();

    // let pcs_srs = {
    //     match read_srs() {
    //         Ok(p) => p,
    //         Err(_e) => {
    //             let srs =
    //                 MultilinearKzgPCS::<Bls12_381>::gen_srs_for_testing(&mut rng, SUPPORTED_SIZE)?;
    //             write_srs(&srs);
    //             srs
    //         },
    //     }
    // };
    // bench_jellyfish_plonk(&pcs_srs, thread)?;
    // println!();
    // bench_vanilla_plonk(&pcs_srs, thread)?;
    // println!();
    // for degree in MIN_CUSTOM_DEGREE..=MAX_CUSTOM_DEGREE {
    //     bench_high_degree_plonk(&pcs_srs, degree, thread)?;
    //     println!();
    // }
    println!();

    Ok(())
}

// fn read_srs() -> Result<MultilinearUniversalParams<Bls12_381>, io::Error> {
//     let mut f = File::open("srs.params")?;
//     Ok(MultilinearUniversalParams::<Bls12_381>::deserialize_compressed_unchecked(&mut f).unwrap())
// }
//
// fn write_srs(pcs_srs: &MultilinearUniversalParams<Bls12_381>) {
//     let mut f = File::create("srs.params").unwrap();
//     pcs_srs.serialize_uncompressed(&mut f).unwrap();
// }
//
// fn bench_vanilla_plonk(
//     pcs_srs: &MultilinearUniversalParams<Bls12_381>,
//     thread: usize,
// ) -> Result<(), HyperPlonkErrors> {
//     let filename = format!("vanilla threads {}.txt", thread);
//     let mut file = File::create(filename).unwrap();
//     for nv in MIN_NUM_VARS..=MAX_NUM_VARS {
//         let vanilla_gate = CustomizedGates::vanilla_plonk_gate();
//         bench_mock_circuit_zkp_helper(&mut file, nv, &vanilla_gate, pcs_srs)?;
//     }
//
//     Ok(())
// }
//
// fn bench_jellyfish_plonk(
//     pcs_srs: &MultilinearUniversalParams<Bls12_381>,
//     thread: usize,
// ) -> Result<(), HyperPlonkErrors> {
//     let filename = format!("jellyfish threads {}.txt", thread);
//     let mut file = File::create(filename).unwrap();
//     for nv in MIN_NUM_VARS..=MAX_NUM_VARS {
//         let jf_gate = CustomizedGates::jellyfish_turbo_plonk_gate();
//         bench_mock_circuit_zkp_helper(&mut file, nv, &jf_gate, pcs_srs)?;
//     }
//
//     Ok(())
// }
//
// fn bench_high_degree_plonk(
//     pcs_srs: &MultilinearUniversalParams<Bls12_381>,
//     degree: usize,
//     thread: usize,
// ) -> Result<(), HyperPlonkErrors> {
//     let filename = format!("high degree {} thread {}.txt", degree, thread);
//     let mut file = File::create(filename).unwrap();
//     println!("custom gate of degree {}", degree);
//     let vanilla_gate = CustomizedGates::mock_gate(2, degree);
//     bench_mock_circuit_zkp_helper(&mut file, HIGH_DEGREE_TEST_NV, &vanilla_gate, pcs_srs)?;
//
//     Ok(())
// }
//
// fn bench_mock_circuit_zkp_helper(
//     file: &mut File,
//     nv: usize,
//     gate: &CustomizedGates,
//     pcs_srs: &MultilinearUniversalParams<Bls12_381>,
// ) -> Result<(), HyperPlonkErrors> {
//     let repetition = if nv < 10 {
//         5
//     } else if nv < 20 {
//         2
//     } else {
//         1
//     };
//
//     //==========================================================
//     let circuit = MockCircuit::<Fr>::new(1 << nv, gate);
//     assert!(circuit.is_satisfied());
//     let index = circuit.index;
//     //==========================================================
//     // generate pk and vks
//     let start = Instant::now();
//     for _ in 0..repetition {
//         let (_pk, _vk) = <PolyIOP<Fr> as HyperPlonkSNARK<
//             Bls12_381,
//             MultilinearKzgPCS<Bls12_381>,
//         >>::preprocess(&index, pcs_srs)?;
//     }
//     println!(
//         "key extraction for {} variables: {} us",
//         nv,
//         start.elapsed().as_micros() / repetition as u128
//     );
//     let (pk, vk) =
//         <PolyIOP<Fr> as HyperPlonkSNARK<Bls12_381, MultilinearKzgPCS<Bls12_381>>>::preprocess(
//             &index, pcs_srs,
//         )?;
//     //==========================================================
//     // generate a proof
//     let start = Instant::now();
//     for _ in 0..repetition {
//         let _proof =
//             <PolyIOP<Fr> as HyperPlonkSNARK<Bls12_381, MultilinearKzgPCS<Bls12_381>>>::prove(
//                 &pk,
//                 &circuit.public_inputs,
//                 &circuit.witnesses,
//             )?;
//     }
//     let t = start.elapsed().as_micros() / repetition as u128;
//     println!(
//         "proving for {} variables: {} us",
//         nv,
//         start.elapsed().as_micros() / repetition as u128
//     );
//     file.write_all(format!("{} {}\n", nv, t).as_ref()).unwrap();
//
//     let proof = <PolyIOP<Fr> as HyperPlonkSNARK<Bls12_381, MultilinearKzgPCS<Bls12_381>>>::prove(
//         &pk,
//         &circuit.public_inputs,
//         &circuit.witnesses,
//     )?;
//     //==========================================================
//     // verify a proof
//     let start = Instant::now();
//     for _ in 0..repetition {
//         let verify =
//             <PolyIOP<Fr> as HyperPlonkSNARK<Bls12_381, MultilinearKzgPCS<Bls12_381>>>::verify(
//                 &vk,
//                 &circuit.public_inputs,
//                 &proof,
//             )?;
//         assert!(verify);
//     }
//     println!(
//         "verifying for {} variables: {} us",
//         nv,
//         start.elapsed().as_micros() / repetition as u128
//     );
//     Ok(())
// }

fn test_hyperplonk_e2e() -> Result<(), HyperPlonkErrors> {
    let mock_gate = CustomizedGates::vanilla_plonk_gate();
    println!("---------begin test mecury---------");
    test_hyperplonk_helper::<Bn254>(mock_gate.clone());
    println!("---------finish test mecury---------");
    println!("---------begin test sama---------");
    test_hyperplonk_Sama::<Bn254>(mock_gate)
}

fn test_hyperplonk_helper<E: Pairing>(
    mock_gate: CustomizedGates,
) -> Result<(), HyperPlonkErrors> {

    let mut rng = test_rng();
    let start = Instant::now();
    let pcs_srs =  MercuryPCS::<E>::gen_srs_for_testing(&mut rng, 16)?;
    let duration = start.elapsed();
    println!("-----------------Setup Mercury Duration{:?}",duration);
    let start = Instant::now();
    let num_constraints = 1 << 8;
    let num_partitions = 2;
    // let num_witness = 5;
    // let degree = 4;

    let partition_circuits = MockCircuit::<E::ScalarField>::partition_circuit::<StdRng>(
        num_constraints,
        &mock_gate,
        num_partitions,
    );

    let duration_sel = start.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let (pk, vk) = <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E,  MercuryPCS<E>>>::preprocess(
        &partition_circuits[0].index,
        &pcs_srs,
    )?;
    let prove= Instant::now();
    let (f_hats, perm_f_hats, f_hat_commitments, perm_f_commitments,duration_wit) =
        <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E,  MercuryPCS<E>>>::mul_prove(
            &pk,
            partition_circuits,
        )?;
    let duration_com = duration_sel + duration_wit;
    println!("-----------------Commit Mercury Duration {:?}",duration_com);
    let sums = vec![E::ScalarField::zero(); f_hats.len()];

    let start = Instant::now();
    let (q_proof, q_sum, q_aux_info, fold_poly, fold_sum) =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::sum_fold(
            f_hats.clone(),
            sums,
            &mut transcript,
        )?;
    let duration_fold1 = start.elapsed();

    let  start = Instant::now();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        q_sum,
        &q_proof,
        &q_aux_info,
        &mut transcript,
    )?;
    let duration_verify1 = start.elapsed();

    let start= Instant::now();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let proof = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::prove(
        &fold_poly.deep_copy(),
        &mut transcript,
    )?;
    let duration_check1 = start.elapsed();

    let start= Instant::now();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        fold_sum,
        &proof,
        &fold_poly.aux_info,
        &mut transcript,
    )?;
    // assert!(
    //     fold_poly.evaluate(&subclaim.point).unwrap() == subclaim.expected_evaluation,
    //     "wrong subclaim f_hats"
    // );
    let duration_verify2 = start.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let sums = vec![E::ScalarField::zero(); perm_f_hats.len()];

    let start= Instant::now();
    let (perm_q_proof, perm_q_sum, perm_q_aux_info, perm_fold_poly, perm_fold_sum) =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::sum_fold(
            perm_f_hats.clone(),
            sums,
            &mut transcript,
        )?;
    let duration_fold2 = start.elapsed();
    // 验证 perm_f_hats 的求和检查子声明
    let start= Instant::now();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let perm_subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        perm_q_sum,
        &perm_q_proof,
        &perm_q_aux_info,
        &mut transcript,
    )?;
    let duration_verify3 = start.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let start= Instant::now();
    let perm_proof = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::prove(
        &perm_fold_poly.deep_copy(),
        &mut transcript,
    )?;
    let duration_check2 = start.elapsed();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let start= Instant::now();
    let perm_subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        fold_sum,
        &perm_proof,
        &perm_fold_poly.aux_info,
        &mut transcript,
    )?;

    let duration_verify4 = start.elapsed();

    let sumcheck_fold_duration = duration_fold1 + duration_fold2;
    let sumcheck_prove_duration = duration_check1+duration_check2;
    let sumcheck_verify_duration = duration_verify1+duration_verify2+duration_verify3+duration_verify4;
    println!("----------SumFold Duration------------{:?}",sumcheck_fold_duration);
    println!("----------SumCheck Prove Duration----------{:?}",sumcheck_prove_duration);
    println!("----------SumCheck Verify Duration---------{:?}",sumcheck_verify_duration);

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let (f_folded_evals, perm_folded_evals, batch_opening_proof) =
        <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E,  MercuryPCS<E>>>::prove(
            f_hats.clone(),
            perm_f_hats.clone(),
            f_hat_commitments.clone(),
            perm_f_commitments.clone(),
            &q_proof,
            &perm_q_proof,
            &pk,
            &mut transcript,
        )?;

    let polys = vec![(f_hats, f_folded_evals), (perm_f_hats, perm_folded_evals)];
    let commitments = [f_hat_commitments, perm_f_commitments].concat();
    let q_proofs = vec![q_proof, perm_q_proof];
    let prove_duration = prove.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let start= Instant::now();
    let is_valid = <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E,  MercuryPCS<E>>>::verify(
        polys,
        commitments,
        q_proofs,
        batch_opening_proof,
        &vk,
        &mut transcript,
    )?;
    assert!(is_valid, "HyperPlonk verification failed");
    let duration = start.elapsed();
    println!("--------------Verify Duration----------------{:?}",duration);
    Ok(())
}

// #[test]
// fn test_hyperplonk_Samaritan() -> Result<(), HyperPlonkErrors> {
//     let mock_gate = CustomizedGates::vanilla_plonk_gate();

//     test_hyperplonk_Sama::<Bls12_381>(mock_gate)
// }

fn test_hyperplonk_Sama<E: Pairing>(
    mock_gate: CustomizedGates,
) -> Result<(), HyperPlonkErrors> {
    let mut rng = test_rng();
    let start = Instant::now();
    let pcs_srs =  SamaritanPCS::<E>::gen_srs_for_testing(&mut rng, 16)?;
    let duration = start.elapsed();
    println!("-----------------Setup Samaritan Duration{:?}",duration);
    let start = Instant::now();
    let num_constraints = 1 << 8;
    let num_partitions = 2;
    // let num_witness = 5;
    // let degree = 4;

    let partition_circuits = MockCircuit::<E::ScalarField>::partition_circuit::<StdRng>(
        num_constraints,
        &mock_gate,
        num_partitions,
    );

    let duration_sel = start.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let (pk, vk) = <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E,  SamaritanPCS<E>>>::preprocess(
        &partition_circuits[0].index,
        &pcs_srs,
    )?;
    let prove= Instant::now();
    let (f_hats, perm_f_hats, f_hat_commitments, perm_f_commitments,duration_wit) =
        <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E, SamaritanPCS<E>>>::mul_prove(
            &pk,
            partition_circuits,
        )?;
    let duration_com = duration_sel + duration_wit;
    println!("-----------------Commit Samaritan Duration{:?}",duration_com);
    let sums = vec![E::ScalarField::zero(); f_hats.len()];

    let start = Instant::now();
    let (q_proof, q_sum, q_aux_info, fold_poly, fold_sum) =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::sum_fold(
            f_hats.clone(),
            sums,
            &mut transcript,
        )?;
    let duration_fold1 = start.elapsed();

    let  start = Instant::now();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        q_sum,
        &q_proof,
        &q_aux_info,
        &mut transcript,
    )?;
    let duration_verify1 = start.elapsed();

    let start= Instant::now();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let proof = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::prove(
        &fold_poly.deep_copy(),
        &mut transcript,
    )?;
    let duration_check1 = start.elapsed();

    let start= Instant::now();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        fold_sum,
        &proof,
        &fold_poly.aux_info,
        &mut transcript,
    )?;
    // assert!(
    //     fold_poly.evaluate(&subclaim.point).unwrap() == subclaim.expected_evaluation,
    //     "wrong subclaim f_hats"
    // );
    let duration_verify2 = start.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let sums = vec![E::ScalarField::zero(); perm_f_hats.len()];

    let start= Instant::now();
    let (perm_q_proof, perm_q_sum, perm_q_aux_info, perm_fold_poly, perm_fold_sum) =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::sum_fold(
            perm_f_hats.clone(),
            sums,
            &mut transcript,
        )?;
    let duration_fold2 = start.elapsed();
    // 验证 perm_f_hats 的求和检查子声明
    let start= Instant::now();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();
    let perm_subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        perm_q_sum,
        &perm_q_proof,
        &perm_q_aux_info,
        &mut transcript,
    )?;
    let duration_verify3 = start.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let start= Instant::now();
    let perm_proof = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::prove(
        &perm_fold_poly.deep_copy(),
        &mut transcript,
    )?;
    let duration_check2 = start.elapsed();
    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let start= Instant::now();
    let perm_subclaim = <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::verify(
        fold_sum,
        &perm_proof,
        &perm_fold_poly.aux_info,
        &mut transcript,
    )?;

    let duration_verify4 = start.elapsed();

    let sumcheck_fold_duration = duration_fold1 + duration_fold2;
    let sumcheck_prove_duration = duration_check1+duration_check2;
    let sumcheck_verify_duration = duration_verify1+duration_verify2+duration_verify3+duration_verify4;
    println!("----------SumFold Duration------------{:?}",sumcheck_fold_duration);
    println!("----------SumCheck Prove Duration----------{:?}",sumcheck_prove_duration);
    println!("----------SumCheck Verify Duration---------{:?}",sumcheck_verify_duration);

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let (f_folded_evals, perm_folded_evals, batch_opening_proof) =
        <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E, SamaritanPCS<E>>>::prove(
            f_hats.clone(),
            perm_f_hats.clone(),
            f_hat_commitments.clone(),
            perm_f_commitments.clone(),
            &q_proof,
            &perm_q_proof,
            &pk,
            &mut transcript,
        )?;

    let polys = vec![(f_hats, f_folded_evals), (perm_f_hats, perm_folded_evals)];
    let commitments = [f_hat_commitments, perm_f_commitments].concat();
    let q_proofs = vec![q_proof, perm_q_proof];
    let prove_duration = prove.elapsed();

    let mut transcript =
        <PolyIOP<E::ScalarField> as SumCheck<E::ScalarField>>::init_transcript();

    let start= Instant::now();
    let is_valid = <PolyIOP<E::ScalarField> as HyperPlonkSNARK<E, SamaritanPCS<E>>>::verify(
        polys,
        commitments,
        q_proofs,
        batch_opening_proof,
        &vk,
        &mut transcript,
    )?;
    assert!(is_valid, "HyperPlonk verification failed");
    let duration = start.elapsed();
    println!("--------------Verify Duration----------------{:?}",duration);
    Ok(())
}