use std::ops::{Add, Mul, Sub};
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_poly::Polynomial;
use ark_std::{UniformRand};
use rand::thread_rng;
use crate::field::Field64 as F;
use crate::polynomial::{LinearDescription, MLPolynomial};
use crate::protocol::rejection::RejectError;

pub struct VerifierState {
    pub last_round: usize,
    pub poly: SparsePolynomial<F, SparseTerm>,
    pub claimed_sum: F,
    pub running_eval: F,
    pub randomness: Vec<F>,
}

pub struct Verifier{
}

impl Verifier {
    pub fn initialize(poly: &MLPolynomial, claimed: F) -> VerifierState {
        VerifierState{
            last_round: 0,
            poly: poly.clone(),
            claimed_sum: claimed,
            running_eval: claimed,
            randomness: Vec::new(),
        }
    }

    pub fn round(state: VerifierState, lin_desc: LinearDescription) -> Result<(F, VerifierState), RejectError> {
        if Self::evaluate_intermediate(&lin_desc).ne(&state.running_eval) {
            return Err(RejectError::new("Rejecting the Prover's claim!"));
        }
        let mut rng = thread_rng();
        let r = F::rand(&mut rng);
        let mut new_rand = state.randomness.clone();
        new_rand.push(r);
        let new_state = VerifierState{
            last_round: state.last_round + 1,
            poly: state.poly,
            claimed_sum: state.claimed_sum,
            running_eval: Self::evaluate_at_random_point(&lin_desc, r),
            randomness: new_rand,
        };
        return Ok((r, new_state))
    }

    pub fn evaluate_intermediate((p0, p1): &LinearDescription) -> F{
        p0.add(p1)
    }

    pub fn evaluate_at_random_point((p0, p1): &LinearDescription, r: F) -> F{
        p0.sub(&r.mul(p0)).add(&r.mul(p1))
    }


    pub fn sanity_check(state: VerifierState) -> (bool, Vec<F>) {
        (state.poly.evaluate(&state.randomness).eq(&state.running_eval), state.randomness)
    }
}

#[cfg(test)]
mod tests {
    use ark_poly::DenseMVPolynomial;
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
    use crate::protocol::prover::Prover;
    use crate::protocol::setup_protocol;
    use super::*;

    #[test]
    fn test_evaluate_intermediate() {
        let poly = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(7), SparseTerm::new(vec![(0,1), (1,1)])),
                (F::from(42), SparseTerm::new(vec![])),
            ])
        );

        let (_, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);

        assert_eq!(claimed_sum, F::from(179));
        assert_eq!(verifier_state.running_eval, claimed_sum);

        let (poly_descr, _) = Prover::round_phase_1(prover_state);
        let expected: LinearDescription = (F::from(85), F::from(94));
        assert_eq!(poly_descr, expected);
        let evaluation = Verifier::evaluate_intermediate(&poly_descr);
        assert_eq!(evaluation, verifier_state.running_eval)
    }
}