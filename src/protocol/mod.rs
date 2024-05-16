use ark_poly::{multivariate::{SparsePolynomial, SparseTerm, Term}, DenseMVPolynomial};
use crate::field::Field64 as F;
use crate::protocol::prover::{Prover, ProverState};
use crate::protocol::verifier::{Verifier, VerifierState};

mod prover;
mod verifier;
mod rejection;

fn orchestrate_protocol(poly: &SparsePolynomial<F, SparseTerm>) -> bool {
    let mut prover_state: ProverState;
    let mut verifier_state: VerifierState;
    let (mut p0, mut p1): (F,F);
    let claimed_sum: F;
    (claimed_sum, prover_state) = Prover::claim_sum(poly);
    verifier_state = Verifier::initialize(poly, claimed_sum);
    let mut r: F;
    for _ in 0..poly.num_vars
    {
        ((p0, p1), prover_state) = Prover::round_phase_1(prover_state);
        (r, verifier_state) = Verifier::round(verifier_state, (p0, p1)).unwrap();
        prover_state = Prover::round_phase_2(prover_state, r);
    }
    Verifier::sanity_check(verifier_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol() {

        let poly = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );

        assert!(orchestrate_protocol(&poly));
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
        assert!(orchestrate_protocol(&poly));

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
        assert!(orchestrate_protocol(&poly));
    }
}