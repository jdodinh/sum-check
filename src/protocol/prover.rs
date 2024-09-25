use std::collections::HashMap;
use std::ops::{Add, Mul, Sub};
use ark_ff::Field;
use ark_poly::multivariate::{SparsePolynomial, SparseTerm};
use ark_std::iterable::Iterable;
use crate::field::Field256 as F;
use crate::polynomial::*;

pub struct ProverState {
    last_round: usize,
    num_vars: usize,
    num_polys: usize,
    maps: Vec<EvalTable>,
}


pub struct Prover {
}

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
        for b in 0..2_u64.pow(num_vars as u32) {
            let b_str = number_to_bit_string(b, num_vars);
            product = initial_state.maps.iter().map(|m|m.get(&b_str).unwrap()).fold(F::ONE, F::mul);
            claim = claim.add(product)
        }
        return (claim, initial_state)

    }

    pub fn round_phase_1(state: ProverState) -> (PolynomialDescription, ProverState) {
        let num_vars = state.num_vars - state.last_round - 1;
        let mut polynomial_points: PolynomialDescription = vec![F::ZERO; state.num_polys+1];
        let mut b_str: String;
        for b in 0..2_u64.pow(num_vars as u32) {
            b_str = number_to_bit_string(b, num_vars);
            polynomial_points = polynomial_points
                .iter()
                .zip(Self::get_polynomial_points(&state, &b_str).iter())
                .map(|(&b, &v)| b.add(v))
                .collect();
        }
        return (polynomial_points, state);
    }

    fn get_polynomial_points(state: &ProverState, b_str: &String) -> PolynomialDescription {
        let mut poly_description: PolynomialDescription = vec![F::ONE; state.num_polys+1];
        for k in 0..state.num_polys {
            poly_description = poly_description
                .iter()
                .zip(Self::get_polynomial_descr_points(state.maps.get(k).unwrap(), &b_str, state.num_polys).iter())
                .map(|(&b, &v)| b.mul(v))
                .collect();
        }
        poly_description
    }

    fn get_polynomial_descr_points(eval_table: &EvalTable, b_str: &String, num_polys: usize) -> PolynomialDescription {
        let mut points: PolynomialDescription = Vec::new();
        let mut t0: &F;
        let mut t1: &F;
        let mut jf: F;
        for j in 0..num_polys + 1 {
            t0 = eval_table.get(&format!("0{b_str}")).unwrap();
            t1 = eval_table.get(&format!("1{b_str}")).unwrap();
            jf = F::from(j as u16);
            points.push(t0.sub(&jf.mul(t0)).add(&jf.mul(t1)))
        }
        points
    }

    pub fn round_phase_2(state: ProverState, r: F) -> ProverState{
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
    tables.iter().map(|table|reduce_map(num_vars, r, table)).collect()
}

fn reduce_map(num_vars: usize, r: F, map: &HashMap<String, F>) -> EvalTable {
    (0..(2_u64.pow(num_vars as u32)))
        .map(|n|number_to_bit_string(n, num_vars))
        .map(|bit_string| (bit_string.clone(), combine_table_elements(bit_string, r, map)))
        .collect::<HashMap<String, F>>()
}

fn combine_table_elements(bit_string: String, r: F, table: &EvalTable) -> F {
    let a0 = table.get(&format!("0{bit_string}")).unwrap();
    let a1 = table.get(&format!("1{bit_string}")).unwrap();
    return a0.sub(&r.mul(a0)).add(&r.mul(a1))
}

fn collapse_map(map: &HashMap<String, F>) -> (F, F) {
    map.iter()
        .fold((F::from(0), F::from(0)), |acc, kv| add_to_acc(acc, kv))
}

fn add_to_acc((acc0, acc1): (F, F), (k, v): (&String, &F)) -> (F, F){
    if k.starts_with("0") {
        (acc0.add(v), acc1)
    } else {
        (acc0, acc1.add(v))
    }
}

#[cfg(test)]
mod tests {
    use ark_poly::DenseMVPolynomial;
    use ark_poly::multivariate::Term;
    use super::*;

    #[test]
    fn test_collapse_map() {
        let our_map = HashMap::from([
            ("000".to_owned(), F::from(67)),
            ("001".to_owned(), F::from(9)),
            ("010".to_owned(), F::from(28)),
            ("011".to_owned(), F::from(31)),
            ("100".to_owned(), F::from(93)),
            ("101".to_owned(), F::from(21)),
            ("110".to_owned(), F::from(72)),
            ("111".to_owned(), F::from(95)),
        ]);
        let p = collapse_map(&our_map);
        assert_eq!(p, (F::from(135), F::from(281)))
    }

    #[test]
    fn test_reduce_map() {
        let our_map = HashMap::from([
            ("000".to_owned(), F::from(67)),
            ("001".to_owned(), F::from(9)),
            ("010".to_owned(), F::from(28)),
            ("011".to_owned(), F::from(31)),
            ("100".to_owned(), F::from(93)),
            ("101".to_owned(), F::from(21)),
            ("110".to_owned(), F::from(72)),
            ("111".to_owned(), F::from(95)),
        ]);
        let r = F::from(83);
        let reduced = reduce_map(2, r, &our_map);
        let expected = HashMap::from([
            ("00".to_owned(), F::from(2225)),
            ("01".to_owned(), F::from(1005)),
            ("10".to_owned(), F::from(3680)),
            ("11".to_owned(), F::from(5343)),
        ]);

        assert!(reduced.eq(&expected));
    }


    #[test]
    fn test_claimed_sum_1() {
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
        let (prover_claim, prover_state) = Prover::claim_sum(&multilinear_list);
        assert_eq!(prover_claim, F::from(93));
        let (poly_descr, _) = Prover::round_phase_1(prover_state);
        let expected: PolynomialDescription = Vec::from([
            F::from(21),
            F::from(72),
            F::from(135),
            F::from(210),]);
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
        let multilinear_list = vec![
            p1, p2
        ];
        let (prover_claim, prover_state) = Prover::claim_sum(&multilinear_list);
        assert_eq!(prover_claim, F::from(24));
        let (poly_descr, _) = Prover::round_phase_1(prover_state);
        let expected: PolynomialDescription = Vec::from([
            F::from(6),
            F::from(18),
            F::from(38),
        ]);
        assert_eq!(poly_descr, expected)
    }
}
