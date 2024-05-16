use std::collections::HashMap;
use std::ops::{Add, Mul, Sub};
use ark_poly::{multivariate::{SparsePolynomial, SparseTerm, Term}, DenseMVPolynomial};
use crate::field::Field64 as F;
use crate::polynomial::*;

pub struct ProverState {
    last_round: usize,
    poly: SparsePolynomial<F, SparseTerm>,
    map: HashMap<String, F>,
}

pub struct Prover {
}

impl Prover {

    pub fn claim_sum(poly: &SparsePolynomial<F, SparseTerm>) -> (F, ProverState) {
        let initial_state = ProverState {
            last_round: 0,
            poly: poly.clone(),
            map: evaluate_polynomial_on_hypercube(poly),
        };
        let claim = initial_state.map.iter().fold(F::from(0), |acc, (_, v)| acc.add(v));
        return (claim, initial_state)

    }

    pub fn round_phase_1(state: ProverState) -> ((F, F), ProverState) {
        let (p0, p1) = collapse_map(&state.map);
        ((p0, p1), state)
    }

    pub fn round_phase_2(state: ProverState, r: F) -> ProverState {
        let num_vars = state.poly.num_vars - state.last_round - 1;
        let new_map = reduce_map(num_vars, r, &state.map);
        let new_state = ProverState {
            last_round: state.last_round + 1,
            poly: state.poly,
            map: new_map,
        };
        new_state
    }
}

fn reduce_map(num_vars: usize, r: F, map: &HashMap<String, F>) -> HashMap<String, F> {
    (0..(2_u64.pow(num_vars as u32)))
        .map(|n|number_to_bit_string(n, num_vars))
        .map(|bit_string| (bit_string.clone(), combine_table_elements(bit_string, r, map)))
        .collect::<HashMap<String, F>>()
}

fn combine_table_elements(bit_string: String, r: F, map: &HashMap<String, F>) -> F {
    let a0 = map.get(&format!("0{bit_string}")).unwrap();
    let a1 = map.get(&format!("1{bit_string}")).unwrap();
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
}