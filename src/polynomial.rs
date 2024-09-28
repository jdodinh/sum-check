use ark_ff::Field;
use ark_poly::{
    multivariate::{SparsePolynomial, SparseTerm},
    DenseMVPolynomial, Polynomial,
};
use std::ops::Mul;

use crate::field::Field256 as F;

/// Type for a multilinear polynomial.
pub type MLPolynomial = SparsePolynomial<F, SparseTerm>;

/// Type for a product of multilinear polynomials.
pub type ProductMLPolynomial = Vec<MLPolynomial>;

/// 'Enough' evaluation points of a univariate polynomial for perfect Lagrange interpolation.
pub type PolynomialDescription = Vec<F>;

/// Type for the evaluation table of a polynomial.
pub type EvalTable = Vec<F>;

/// Evaluates a ProductMLPolynomial at 'point'
pub fn evaluate_mvml_polynomial(mvml_polynomial: ProductMLPolynomial, point: &Vec<F>) -> F {
    mvml_polynomial
        .iter()
        .map(|ml_polynomial| ml_polynomial.evaluate(&point))
        .fold(F::ONE, F::mul)
}

/// Returns an optional number of variables in a ProductMLPolynomial. Is None if number of variables
/// is not the same in each polynomial.
pub fn get_num_vars(multilinears: &ProductMLPolynomial) -> Option<usize> {
    match multilinears.as_slice() {
        [head, tail @ ..] => tail
            .iter()
            .all(|x| x.num_vars == head.num_vars)
            .then(|| head.num_vars),
        [] => None,
    }
}

/// Obtain the evaluation table on the binary hypercube for a multilinear polynomial.
pub fn evaluate_polynomial_on_hypercube(p: &MLPolynomial) -> EvalTable {
    let num_vars = p.num_vars();
    (0..(1 << num_vars) as usize)
        .map(|n| usize_to_binary_vector(n, num_vars))
        .map(|binary| p.evaluate(&binary))
        .collect::<Vec<F>>()
}

fn usize_to_binary_vector(n: usize, num_vars: usize) -> Vec<F> {
    let mut result = Vec::with_capacity(64);
    for i in (0..64).rev() {
        result.push(if (n & (1 << i)) != 0 { F::ONE } else { F::ZERO });
    }
    result.split_off(64 - num_vars)
}

#[cfg(test)]
mod tests {

    use super::*;
    use ark_ff::Field;
    use ark_poly::multivariate::Term;
    use ark_poly::Polynomial;
    use ark_std::UniformRand;
    use rand::thread_rng;

    #[test]
    fn test_polynomial_equality() {
        let poly1 = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 3)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );

        let poly2 = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 3)])),
                (F::from(1), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(6), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );

        let mut rng = thread_rng();
        let random_point = vec![F::rand(&mut rng), F::rand(&mut rng), F::rand(&mut rng)];

        assert!(poly1.eq(&poly2));
        assert_eq!(poly1.evaluate(&random_point), poly2.evaluate(&random_point));
    }

    #[test]
    fn test_number_to_vector() {
        let point = usize_to_binary_vector(4829, 16);
        assert_eq!(point.len(), 16);

        let point = usize_to_binary_vector(4, 5);
        assert_eq!(point, vec![F::ZERO, F::ZERO, F::ONE, F::ZERO, F::ZERO,]);

        let point = usize_to_binary_vector(53, 6);
        assert_eq!(
            point,
            vec![F::ONE, F::ONE, F::ZERO, F::ONE, F::ZERO, F::ONE,]
        );

        let num_vars: u32 = 10;
        let point = usize_to_binary_vector((1 << num_vars as usize) - 1, 11);
        assert_eq!(
            point,
            vec![
                F::ZERO,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
                F::ONE,
            ]
        );
    }

    #[test]
    fn test_evaluate_polynomial() {
        let poly = SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 3)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        );

        let map = evaluate_polynomial_on_hypercube(&poly);
        let point: usize = 3;
        let some_point = vec![F::ZERO, F::ONE, F::ONE];
        let value_from_map = map.get(point).unwrap();
        let value_from_poly = poly.evaluate(&some_point);
        assert_eq!(map.len(), 8);
        assert_eq!(some_point, usize_to_binary_vector(point, 3));
        assert_eq!(*value_from_map, value_from_poly)
    }
}
