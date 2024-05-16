use std::ops::{Add, Mul, Sub};
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_poly::Polynomial;
use ark_std::{UniformRand};
use rand::thread_rng;
use crate::field::Field64 as F;
use crate::protocol::rejection::RejectError;

pub struct VerifierState {
    last_round: usize,
    poly: SparsePolynomial<F, SparseTerm>,
    claimed_sum: F,
    running_eval: F,
    randomness: Vec<F>,
}

pub struct Verifier{
}

impl Verifier {
    pub fn initialize(poly: &SparsePolynomial<F, SparseTerm>, claimed: F) -> VerifierState {
        VerifierState{
            last_round: 0,
            poly: poly.clone(),
            claimed_sum: claimed,
            running_eval: claimed,
            randomness: Vec::new(),
        }
    }

    pub fn round(state: VerifierState, (p0, p1): (F, F)) -> Result<(F, VerifierState), RejectError> {
        if p0.add(p1).ne(&state.running_eval) {
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
            running_eval: p0.sub(&r.mul(p0)).add(&r.mul(p1)),
            randomness: new_rand,
        };
        return Ok((r, new_state))
    }

    pub fn sanity_check(state: VerifierState) -> bool {
        state.poly.evaluate(&state.randomness).eq(&state.running_eval)
    }
}