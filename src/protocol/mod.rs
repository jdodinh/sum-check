use ark_poly::{multivariate::{SparsePolynomial, SparseTerm, Term}, DenseMVPolynomial};
use crate::field::Field64 as F;
use crate::polynomial::{get_num_vars, MVMLDescription, ProductPolynomial};
use crate::protocol::prover::{Prover, ProverState};
use crate::protocol::verifier::{Verifier, VerifierState};

mod prover;
mod verifier;
mod rejection;

struct ProtocolTranscript {
    randomness: Vec<F>,
    claimed_sum: F,
    accept: bool,
}

fn setup_protocol(poly: &ProductPolynomial) -> (usize, F, ProverState, VerifierState) {
    let num_vars = get_num_vars(poly.as_slice()).unwrap();
    let (claimed_sum, prover_state) = Prover::claim_sum(poly);
    let verifier_state = Verifier::initialize(poly, claimed_sum);
    (num_vars, claimed_sum, prover_state, verifier_state)
}

fn orchestrate_protocol(num_vars: usize,
                        claimed_sum: F,
                        mut prover_state: ProverState,
                        mut verifier_state: VerifierState)
    -> ProtocolTranscript {
    let mut poly_descr: MVMLDescription;
    let mut r: F;
    for _ in 0..num_vars
    {
        (poly_descr, prover_state) = Prover::round_phase_1(prover_state);
        (r, verifier_state) = Verifier::round(verifier_state, poly_descr).unwrap();
        prover_state = Prover::round_phase_2(prover_state, r);
    }
    let (accept, randomness) = Verifier::sanity_check(verifier_state);
    ProtocolTranscript{
        randomness,
        claimed_sum,
        accept
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use ark_std::UniformRand;
    use rand::thread_rng;
    use crate::polynomial::from_multilinears;
    use super::*;

    #[test]
    fn test_protocol() {

        let poly = vec![SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }


    #[test]
    fn test_protocol_6_variables() {
        let poly = vec![SparsePolynomial::from_coefficients_vec(
            6,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 1)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        )];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);

    }

    #[test]
    fn test_protocol_6_variables_random_coefficients() {
        let mut rng = thread_rng();
        let poly = vec![SparsePolynomial::from_coefficients_vec(
            6,
            vec![
                (F::rand(&mut rng), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::rand(&mut rng), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::rand(&mut rng), SparseTerm::new(vec![(0, 1), (5,1), (3, 1)])),
                (F::rand(&mut rng), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        )];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);

    }

    #[test]
    fn test_protocol_univariate() {

        let poly1 = vec![SparsePolynomial::from_coefficients_vec(
            1,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly1);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    #[test]
    fn test_protocol_product_polynomial() {
        let p1 = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![])),
            ])
        );
        let p2 = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)]))
            ])
        );
        let multilinear_list = vec![
            p1, p2
        ];
        let poly = from_multilinears(multilinear_list.as_slice());
        assert!(poly.is_some());
        let poly = poly.unwrap();

        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        assert_eq!(claimed_sum, F::from(47));
        let expected_map = HashMap::from([
            ("00".to_owned(), [F::from(7), F::from(0)].to_vec()),
            ("01".to_owned(), [F::from(7), F::from(1)].to_vec()),
            ("10".to_owned(), [F::from(8), F::from(2)].to_vec()),
            ("11".to_owned(), [F::from(8), F::from(3)].to_vec()),
        ]);
        assert_eq!(prover_state.map, expected_map);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }


}