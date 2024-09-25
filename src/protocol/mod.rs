use crate::field::Field256 as F;
use crate::polynomial::{get_num_vars, PolynomialDescription, ProductMLPolynomial};
use crate::protocol::prover::{Prover, ProverState};
use crate::protocol::verifier::{Verifier, VerifierState};

mod prover;
mod verifier;
mod rejection;


pub struct ProtocolTranscript {
    _randomness: Vec<F>,
    pub accept: bool,
}

pub fn setup_protocol(poly: &ProductMLPolynomial) -> (usize, F, ProverState, VerifierState) {
    let num_vars = get_num_vars(&poly).unwrap();
    let (claimed_sum, prover_state) = Prover::claim_sum(&poly);
    let verifier_state = Verifier::initialize(&poly, claimed_sum);
    (num_vars, claimed_sum, prover_state, verifier_state)
}

pub fn orchestrate_protocol(num_vars: usize,
                        _claimed_sum: F,
                        mut prover_state: ProverState,
                        mut verifier_state: VerifierState)
                        -> ProtocolTranscript {
    let mut poly_descr: PolynomialDescription;
    for _ in 0..num_vars
    {
        (poly_descr, prover_state) = Prover::round_phase_1(prover_state);
        match Verifier::round(verifier_state, poly_descr) {
            Ok((r, state)) => {
                verifier_state = state;
                prover_state = Prover::round_phase_2(prover_state, r) },
            Err(_) => return ProtocolTranscript{ _randomness: vec![], accept: false}
        }
    }
    let (accept, _randomness) = Verifier::sanity_check(verifier_state);
    ProtocolTranscript{
        _randomness,
        accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_poly::{multivariate::{SparsePolynomial, SparseTerm}, DenseMVPolynomial};
    use ark_poly::multivariate::Term;
    /// Basic test for a multilinear polynomial on 3 variables.
    #[test]
    fn test_protocol_3_variables() {
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    /// Failing test for polynomial on 3 variables, where the input is not given as a product of
    /// multilinear polynomials.
    #[test]
    fn test_fail_3_variables() {
        // We create a polynomial of degree 2, not given as a product of multilinears. The verifier
        // will accept all the intermediate rounds, except the last check.
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![(0, 2), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(!transcript.accept);
    }

    /// Test for a multilinear polynomial on 6 variables.
    #[test]
    fn test_protocol_6_variables() {
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            6,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 1)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    /// Failing test for polynomial on 3 variables, where the input is not given as a product of
    /// multilinear polynomials.
    #[test]
    fn test_fail_6_variables() {
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            6,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 4)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(!transcript.accept);
        assert_eq!(transcript._randomness.len(), 6)

    }

    /// Test for a multilinear polynomial on 12 variables.
    #[test]
    fn test_protocol_12_variables() {
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            12,
            vec![
                (F::from(1), SparseTerm::new(vec![(0, 1), (4,1), (3, 1)])),
                (F::from(83), SparseTerm::new(vec![(0, 1), (3,1), (2, 1)])),
                (F::from(62), SparseTerm::new(vec![(0, 1), (5,1), (3, 1)])),
                (F::from(84), SparseTerm::new(vec![(2, 1), (4,1), (3, 1)])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }

    /// Test for a univariate linear polynomial.
    #[test]
    fn test_protocol_univariate() {
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            1,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
        assert_eq!(transcript._randomness.len(), 1)

    }

    /// Failing test for a univariate linear polynomial, where the verifier rejects at an
    /// intermediate round.
    #[test]
    fn test_fail_intermediate_check() {
        let poly = Vec::from(&[SparsePolynomial::from_coefficients_vec(
            1,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )]);
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
        let alt_verifier_state = VerifierState{
            running_eval: F::from(0),
            ..verifier_state
        };
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, alt_verifier_state);
        assert!(!transcript.accept);
        assert_eq!(transcript._randomness.len(), 0)
    }


    /// Test for a polynomial given as a product of multilinear polynomials.
    #[test]
    fn test_product_check() {
        let p1 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let p2 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let p3 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let multilinear_list = vec![
            p1, p2, p3
        ];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&multilinear_list);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(transcript.accept);
    }


    /// Failing test for a polynomial where one of the elements of the products is not multilinear.
    #[test]
    fn test_fail_product_check() {
        let p1 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let p2 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let p3 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 4)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let multilinear_list = vec![
            p1, p2, p3
        ];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&multilinear_list);
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
        assert!(!transcript.accept);
    }

    /// Failing test for a polynomial where the claimed sum is not correct.
    #[test]
    fn test_fail_product_intermediate_check() {
        let p1 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let p2 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let p3 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        );
        let multilinear_list = vec![
            p1, p2, p3
        ];
        let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&multilinear_list);
        let alt_verifier_state = VerifierState{
            running_eval: F::from(0),
            ..verifier_state
        };
        let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, alt_verifier_state);
        assert!(!transcript.accept);
        assert_eq!(transcript._randomness.len(), 0);
    }


}
