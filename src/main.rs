use ark_poly::DenseMVPolynomial;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
use crate::polynomial::ProductMLPolynomial;
use crate::field::Field64 as F;
use crate::protocol::*;

mod protocol;
mod field;
mod polynomial;

fn main() {
    /// The protocol works any time 'poly' is a list of multilinear polynomials. The polynomial used
    /// is the product of these polynomials.
    let poly: ProductMLPolynomial = vec![
        SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1), (2,1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        ),
        SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        ),
        SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ])
        )];
    let (num_vars, claimed_sum, prover_state, verifier_state) = setup_protocol(&poly);
    let transcript = orchestrate_protocol(num_vars, claimed_sum, prover_state, verifier_state);
    if transcript.accept {
        println!("The verifier accepts the claim.");
    } else {
        println!("The verifier accepts the claim.");
    }
}

