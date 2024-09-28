use crate::field::Field256 as F;
use crate::polynomial::*;
use ark_ff::Field;
use ark_std::iterable::Iterable;
use std::ops::{Add, Mul};

pub struct ProverState {
    last_round: usize,
    num_vars: usize,
    num_polys: usize,
    maps: Vec<EvalTable>,
}

pub struct Prover {}

impl Prover {
    pub fn claim_sum(poly: &ProductMLPolynomial) -> (F, ProverState) {
        let num_vars = get_num_vars(&poly).unwrap();
        let initial_state = ProverState {
            last_round: 0,
            num_vars,
            num_polys: poly.len(),
            maps: poly.iter().map(evaluate_polynomial_on_hypercube).collect(),
        };
        let mut claim = F::ZERO;
        let mut product;
        for b in 0..1 << num_vars {
            product = initial_state
                .maps
                .iter()
                .map(|m| m.get(b as usize).unwrap())
                .fold(F::ONE, F::mul);
            claim += product;
        }
        return (claim, initial_state);
    }

    pub fn round_phase_1(state: ProverState) -> (PolynomialDescription, ProverState) {
        let num_vars = state.num_vars - state.last_round - 1;
        let mut polynomial_points: PolynomialDescription = vec![F::ZERO; state.num_polys + 1];
        for b in 0..1 << num_vars {
            polynomial_points = polynomial_points
                .iter()
                .zip(
                    Self::get_polynomial_points(&state, b as usize, (b + (1 << num_vars)) as usize)
                        .iter(),
                )
                .map(|(&b, &v)| b.add(v))
                .collect();
        }
        return (polynomial_points, state);
    }

    fn get_polynomial_points(state: &ProverState, b0: usize, b1: usize) -> PolynomialDescription {
        let mut poly_description: PolynomialDescription = vec![F::ONE; state.num_polys + 1];
        for k in 0..state.num_polys {
            poly_description = poly_description
                .iter()
                .zip(
                    Self::get_polynomial_descr_points(
                        state.maps.get(k).unwrap(),
                        b0,
                        b1,
                        state.num_polys,
                    )
                    .iter(),
                )
                .map(|(&b, &v)| b * v)
                .collect();
        }
        poly_description
    }

    fn get_polynomial_descr_points(
        eval_table: &EvalTable,
        b0: usize,
        b1: usize,
        num_polys: usize,
    ) -> PolynomialDescription {
        let mut points: PolynomialDescription = Vec::new();
        let mut t0: &F;
        let mut t1: &F;
        let mut jf: F;
        for j in 0..=num_polys {
            t0 = eval_table.get(b0).unwrap();
            t1 = eval_table.get(b1).unwrap();
            jf = F::from(j as u16);
            points.push(*t0 - (jf * t0) + (jf * t1))
        }
        points
    }

    pub fn round_phase_2(state: ProverState, r: F) -> ProverState {
        let num_vars = state.num_vars - state.last_round - 1;
        let new_map = reduce(num_vars, r, &state.maps);
        let new_state = ProverState {
            last_round: state.last_round + 1,
            maps: new_map,
            ..state
        };
        new_state
    }
}

fn reduce(num_vars: usize, r: F, tables: &Vec<EvalTable>) -> Vec<EvalTable> {
    tables
        .iter()
        .map(|table| reduce_map(num_vars, r, table))
        .collect()
}

fn reduce_map(num_vars: usize, r: F, map: &Vec<F>) -> EvalTable {
    (0..(1 << num_vars))
        .map(|bit| (combine_table_elements(bit as usize, num_vars, r, map)))
        .collect::<Vec<F>>()
}

fn combine_table_elements(bit: usize, num_vars: usize, r: F, table: &EvalTable) -> F {
    let a0 = table.get(bit).unwrap();
    let a1 = table.get(bit + (1 << num_vars) as usize).unwrap();
    return *a0 - (r * a0) + (r * a1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_poly::multivariate::Term;
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
    use ark_poly::DenseMVPolynomial;

    #[test]
    fn test_reduce_map() {
        let our_map = Vec::from([
            F::from(67),
            F::from(9),
            F::from(28),
            F::from(31),
            F::from(93),
            F::from(21),
            F::from(72),
            F::from(95),
        ]);
        let r = F::from(83);
        let reduced = reduce_map(2, r, &our_map);
        let expected = Vec::from([F::from(2225), F::from(1005), F::from(3680), F::from(5343)]);

        assert!(reduced.eq(&expected));
    }

    #[test]
    fn test_claimed_sum_1() {
        let p1 = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(7), SparseTerm::new(vec![])),
            ]),
        );
        let p2 = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([
                (F::from(2), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
            ]),
        );
        let p3 = SparsePolynomial::from_coefficients_vec(
            2,
            Vec::from([(F::from(3), SparseTerm::new(vec![(1, 1)]))]),
        );
        let multilinear_list = vec![p1, p2, p3];
        let (prover_claim, prover_state) = Prover::claim_sum(&multilinear_list);
        assert_eq!(prover_claim, F::from(93));
        let (poly_descr, _) = Prover::round_phase_1(prover_state);
        let expected: PolynomialDescription =
            Vec::from([F::from(21), F::from(72), F::from(135), F::from(210)]);
        assert_eq!(poly_descr, expected)
    }

    #[test]
    fn test_claimed_sum_2() {
        let p1 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ]),
        );
        let p2 = SparsePolynomial::from_coefficients_vec(
            3,
            Vec::from([
                (F::from(1), SparseTerm::new(vec![(0, 1)])),
                (F::from(1), SparseTerm::new(vec![(1, 1)])),
                (F::from(1), SparseTerm::new(vec![(2, 1)])),
            ]),
        );
        let multilinear_list = vec![p1, p2];
        let (prover_claim, prover_state) = Prover::claim_sum(&multilinear_list);
        assert_eq!(prover_claim, F::from(24));
        let (poly_descr, _) = Prover::round_phase_1(prover_state);
        let expected: PolynomialDescription = Vec::from([F::from(6), F::from(18), F::from(38)]);
        assert_eq!(poly_descr, expected)
    }
}
