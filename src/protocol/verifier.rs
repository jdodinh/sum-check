use std::ops::{Add, Mul, Sub};
use ark_ff::Field;
use ark_poly::Polynomial;
use ark_std::{UniformRand};
use rand::thread_rng;
use crate::field::Field64 as F;
use crate::polynomial::{MVMLDescription, ProductPolynomial};
use crate::protocol::rejection::RejectError;

pub struct VerifierState<'a> {
    last_round: usize,
    poly: &'a ProductPolynomial,
    pub running_eval: F,
    randomness: Vec<F>,
}

pub struct Verifier{
}

impl Verifier {
    pub fn initialize(poly: &ProductPolynomial, claimed: F) -> VerifierState {
        VerifierState{
            last_round: 0,
            poly,
            running_eval: claimed,
            randomness: Vec::new(),
        }
    }

    pub fn round(state: VerifierState, poly_desc: MVMLDescription) -> Result<(F, VerifierState), RejectError> {
        if Self::evaluate_intermediate(&poly_desc).ne(&state.running_eval) {
            return Err(RejectError::new("Rejecting the Prover's claim!"));
        }
        let mut rng = thread_rng();
        let r = F::rand(&mut rng);
        let mut new_rand = state.randomness.clone();
        new_rand.push(r);
        let new_state = VerifierState{
            last_round: state.last_round + 1,
            running_eval: Self::evaluate_at_r(poly_desc, r),
            randomness: new_rand,
            ..state
        };
        return Ok((r, new_state))
    }

    fn evaluate_at_r(desc: MVMLDescription, r: F) -> F {
        let (p0, p1) = desc.iter()
            .map(|(p0, p1)| (p0.sub(&r.mul(p0)), r.mul(p1)))
            .fold((F::ONE, F::ONE), |(acc0, acc1), (e0, e1)| (acc0.mul(e0), acc1.mul(e1)));
        p0.add(p1)
    }

    fn evaluate_intermediate(poly_desc: &MVMLDescription) -> F {
        let (p0, p1) = poly_desc.iter()
            .fold((F::ONE, F::ONE), |(acc0, acc1), (e0, e1)| (acc0.mul(e0), acc1.mul(e1)));
        p0.add(p1)
    }

    pub fn sanity_check(state: VerifierState) -> (bool, Vec<F>) {
        (state.poly.iter().map(|poly| poly.evaluate(&state.randomness)).fold(F::ONE, F::mul)
            .eq(&state.running_eval), state.randomness)
    }
}

#[cfg(test)]
mod tests {
    use ark_poly::DenseMVPolynomial;
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
    use crate::polynomial::from_multilinears;
    use crate::protocol::prover::Prover;
    use crate::protocol::setup_protocol;
    use super::*;

    #[test]
    fn test_evaluate_intermediate() {
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
        assert_eq!(verifier_state.running_eval, claimed_sum);

        let (poly_descr, prover_state) = Prover::round_phase_1(prover_state);
        let expected: MVMLDescription = Vec::from([(F::from(14), F::from(16)), (F::from(1), F::from(5))]);
        assert_eq!(poly_descr, expected);
        let evaluation = Verifier::evaluate_intermediate(&poly_descr);
        assert_eq!(evaluation, verifier_state.running_eval)
    }
}