use std::collections::HashMap;
use ark_ff::Field;
use std::ops::Mul;
use ark_poly::{multivariate::{SparsePolynomial, SparseTerm, Term}, DenseMVPolynomial, Polynomial};

use ark_std::{UniformRand};
use rand::{thread_rng};

use crate::field::Field64 as F;

pub type ProductPolynomial = Vec<SparsePolynomial<F, SparseTerm>>;

pub type MVMLDescription = Vec<(F, F)>;

pub fn from_multilinears(multilinears: &[SparsePolynomial<F, SparseTerm>]) -> Option<ProductPolynomial> {
    match get_num_vars(multilinears) {
        Some(_) => Some(multilinears.to_vec()),
        None => None
    }
}

pub fn get_num_vars(multilinears: &[SparsePolynomial<F, SparseTerm>]) -> Option<usize> {
    match multilinears {
        [head, tail @ ..] => tail
            .iter()
            .all(|x| x.num_vars == head.num_vars &&
                x.degree() == 1 &&
                head.degree() == 1)
            .then(|| head.num_vars),
        [] => None,
    }
}

pub fn evaluate_mvml(product_poly: &ProductPolynomial, point: &Vec<F>) -> F {
    product_poly.iter().map(|ml_poly| ml_poly.evaluate(&point)).fold(F::ONE, F::mul)
}


pub fn evaluate_polynomial_on_hypercube(p: &ProductPolynomial, num_vars: usize) -> HashMap<String, Vec<F>> {
    (0..(2_u64.pow(num_vars as u32)))
        .map(|n|number_to_bit_string(n, num_vars))
        .map(|bit_string| (bit_string.clone(), p.iter().map(|pol| pol.evaluate(&bit_string_to_vector(&bit_string))).collect::<Vec<F>>()))
        .collect::<HashMap<String, Vec<F>>>()
}

pub fn number_to_bit_string(number: u64, num_vars: usize) -> String {
    format!("{number:032b}").split_off(32-num_vars)
}

fn bit_string_to_vector(bit_string: &String) -> Vec<F> {
    bit_string.as_bytes().iter().map(|&b| F::from(b%2)).collect::<Vec<F>>()
}

#[cfg(test)]
mod tests {
    use ark_ff::Field;
    use ark_poly::Polynomial;
    use super::*;

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
        let random_point = vec![
            F::rand(&mut rng),
            F::rand(&mut rng),
            F::rand(&mut rng),
        ];

        assert!(poly1.eq(&poly2));
        assert_eq!(
            poly1.evaluate(&random_point),
            poly2.evaluate(&random_point)
        );

    }

    #[test]
    fn test_number_to_vector() {
        let point = number_to_bit_string(4829, 16);
        assert_eq!(point.len(), 16);

        let point = bit_string_to_vector(&number_to_bit_string(4, 5));
        assert_eq!(point, vec![
            F::ZERO,
            F::ZERO,
            F::ONE,
            F::ZERO,
            F::ZERO,
        ]);

        let point = bit_string_to_vector(&number_to_bit_string(53, 6));
        assert_eq!(point, vec![
            F::ONE,
            F::ONE,
            F::ZERO,
            F::ONE,
            F::ZERO,
            F::ONE,
        ]);

        let num_vars: u32 = 10;
        let point = bit_string_to_vector(&number_to_bit_string(2_u64.pow(num_vars)-1, 11));
        assert_eq!(point, vec![
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
        ]);
    }

    #[test]
    fn test_evaluate_polynomial() {
        let poly = from_multilinears(&[SparsePolynomial::from_coefficients_vec(
            3,
            vec![
                (F::from(2), SparseTerm::new(vec![(0, 3)])),
                (F::from(7), SparseTerm::new(vec![(0, 1), (2, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1), (2, 1)])),
                (F::from(5), SparseTerm::new(vec![])),
            ],
        )]);
        assert!(poly.is_some());
        let poly = poly.unwrap();
        let map = evaluate_polynomial_on_hypercube(&poly, 3);
        let binary_point = String::from("011");
        let some_point = vec![
            F::ZERO,
            F::ONE,
            F::ONE,
        ];
        let value_from_map = map.get(&binary_point).unwrap().iter().fold(F::ONE, F::mul);
        let value_from_poly = evaluate_mvml(&poly, &some_point);
        assert_eq!(map.len(), 8);
        assert_eq!(some_point, bit_string_to_vector(&binary_point));
        assert_eq!(value_from_map, value_from_poly)
    }
    #[test]
    fn test_claimed_sum() {
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
        let p3 = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(3), SparseTerm::new(vec![(1, 1)])),
            ])
        );
        let multilinear_list = vec![
            p1, p2, p3
        ];
        let poly = from_multilinears(multilinear_list.as_slice());
        assert!(poly.is_some());
        let poly = poly.unwrap();

        let some_point = vec![
            F::ONE,
            F::ONE,
        ];
        assert_eq!(evaluate_mvml(&poly, &some_point), F::from(72))
    }

}
