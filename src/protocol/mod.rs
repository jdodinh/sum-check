use ark_poly::{multivariate::{SparsePolynomial, SparseTerm, Term}, DenseMVPolynomial};
use crate::field::Field64 as F;
use crate::polynomial::{LinearDescription, MLPolynomial};
use crate::protocol::prover::{Prover, ProverState};
use crate::protocol::verifier::{Verifier, VerifierState};

mod prover;
mod verifier;
mod rejection;


struct ProtocolTranscript {
    randomness: Vec<F>,
    accept: bool,
}

fn setup_protocol(poly: &MLPolynomial) -> (usize, F, ProverState, VerifierState) {
    let num_vars = poly.num_vars;
    let (claimed_sum, prover_state) = Prover::claim_sum(&poly);
    let verifier_state = Verifier::initialize(&poly, claimed_sum);
    (num_vars, claimed_sum, prover_state, verifier_state)
}

fn orchestrate_protocol(num_vars: usize,
                        _claimed_sum: F,
                        mut prover_state: ProverState,
                        mut verifier_state: VerifierState)
                        -> ProtocolTranscript {
    let mut poly_descr: LinearDescription;
    for _ in 0..num_vars
    {
        (poly_descr, prover_state) = Prover::round_phase_1(prover_state);
        match Verifier::round(verifier_state, poly_descr) {
            Ok((r, state)) => {
                verifier_state = state;
                prover_state = Prover::round_phase_2(prover_state, r) },
            Err(_) => return ProtocolTranscript{ randomness: vec![], accept: false}
        }
    }
    let (accept, randomness) = Verifier::sanity_check(verifier_state);
    ProtocolTranscript{
        randomness,
        accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_3_variables() {

        let poly = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    #[test]
    fn test_fail_3_variables() {
        // We create a polynomial of degree 2. the verifier will accept all the intermediate rounds,
        // except the last check.
        let poly = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![(0, 2), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(!transcript.accept);
    }

    #[test]
    fn test_protocol_6_variables() {
        let poly = SparsePolynomial::from_coefficients_vec(
            6,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 1)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    #[test]
    fn test_fail_6_variables() {
        let poly = SparsePolynomial::from_coefficients_vec(
            6,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 4)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(!transcript.accept);
        assert_eq!(transcript.randomness.len(), 6)

    }

    #[test]
    fn test_protocol_12_variables() {
        let poly = SparsePolynomial::from_coefficients_vec(
            12,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 1)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    #[test]
    fn test_protocol_univariate() {
        let poly = SparsePolynomial::from_coefficients_vec(
            1,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
        assert_eq!(transcript.randomness.len(), 1)

    }

    #[test]
    fn test_fail_intermediate_check() {
        let poly = SparsePolynomial::from_coefficients_vec(
            1,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let alt_verifier_state = VerifierState{
            running_eval: F::from(0),
            ..verifier_state
        };
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, alt_verifier_state);
        assert!(!transcript.accept);
        assert_eq!(transcript.randomness.len(), 0)
    }


}