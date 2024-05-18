use std::ops::{Add, Mul};
use ark_ff::Field;
use ark_std::{UniformRand};
use rand::thread_rng;
use crate::field::Field64 as F;
use crate::polynomial::{evaluate_mvml_polynomial, PolynomialDescription, ProductMLPolynomial};
use crate::protocol::rejection::RejectError;

pub struct VerifierState {
    pub last_round: usize,
    pub num_polys: usize,
    pub poly: ProductMLPolynomial,
    pub claimed_sum: F,
    pub running_eval: F,
    pub randomness: Vec<F>,
}

pub struct Verifier{
}

impl Verifier {
    pub fn initialize(poly: &ProductMLPolynomial, claimed: F) -> VerifierState {
        VerifierState{
            last_round: 0,
            num_polys: poly.len(),
            poly: poly.clone(),
            claimed_sum: claimed,
            running_eval: claimed,
            randomness: Vec::new(),
        }
    }

    /// Execute a round of the verifier. First it checks the consistency with the previous checks,
    /// then generates randomness and returns its updated state, as well as the randomness.
    pub fn round(state: VerifierState, mvml_desc: PolynomialDescription) -> Result<(F, VerifierState), RejectError> {
        if Self::evaluate_intermediate(&mvml_desc).ne(&state.running_eval) {
            return Err(RejectError::new("Rejecting the Prover's claim!"));
        }
        let mut rng = thread_rng();
        let r = F::rand(&mut rng);
        let mut new_rand = state.randomness.clone();
        new_rand.push(r);
        let new_state = VerifierState{
            last_round: state.last_round + 1,
            running_eval: Self::evaluate_at_random_point(&mvml_desc, r),
            randomness: new_rand,
            ..state
        };
        return Ok((r, new_state))
    }

    /// Evaluate p(0) + p(1).
    pub fn evaluate_intermediate(mvml_desc: &PolynomialDescription) -> F{
        mvml_desc.get(0).unwrap().add(mvml_desc.get(1).unwrap())
    }

    /// Evaluate the polynomial at a random point thanks to Lagrange interpolation.
    pub fn evaluate_at_random_point(mvml_descr: &PolynomialDescription, r: F) -> F{
        let k = mvml_descr.len() - 1;
        let mut result = F::ZERO;

        for i in 0..=k {
            let (x_i, y_i) = (F::from(i as u16), mvml_descr[i]);

            // Calculate the Lagrange basis polynomial l_i(r)
            let mut l_i_r = F::ONE;
            for j in 0..=k {
                if i != j {
                    let x_j = F::from(j as u16);
                    l_i_r *= (r - x_j) / (x_i - x_j);
                }
            }

            // Add the term to the result
            result = result.add(y_i.mul(l_i_r));
        }

        result
    }

    /// Last check to see if the polynomial evaluated at a random point agrees with the prover's
    /// messages.
    pub fn sanity_check(state: VerifierState) -> (bool, Vec<F>) {
        (evaluate_mvml_polynomial(state.poly, &state.randomness).eq(&state.running_eval), state.randomness)
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
        let poly = vec![SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(7), SparseTerm::new(vec![(0,1), (1,1)])),
                (F::from(42), SparseTerm::new(vec![])),
            ])
        )];

        let (_, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);

        assert_eq!(claimed_sum, F::from(179));
        assert_eq!(verifier_state.running_eval, claimed_sum);

        let (poly_descr, _) = Prover::round_phase_1(prover_state);
        let expected: PolynomialDescription = vec![F::from(85), F::from(94)];
        assert_eq!(poly_descr, expected);
        let evaluation = Verifier::evaluate_intermediate(&poly_descr);
        assert_eq!(evaluation, verifier_state.running_eval);
        let _ = Verifier::round(verifier_state, poly_descr);
    }
}