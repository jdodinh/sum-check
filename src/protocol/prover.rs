use std::collections::HashMap;
use std::ops::{Add, Mul, Sub};
use ark_ff::Field;
use ark_std::iterable::Iterable;
use crate::field::Field64 as F;
use crate::polynomial::*;

pub struct ProverState {
    last_round: usize,
    num_vars: usize,
    pub map: HashMap<String, Vec<F>>,
}

pub struct Prover {
}

impl Prover {
    pub fn claim_sum(poly: &ProductPolynomial) -> (F, ProverState) {
        let num_vars = get_num_vars(poly.as_slice()).unwrap();
        let initial_state = ProverState {
            last_round: 0,
            num_vars,
            map: evaluate_polynomial_on_hypercube(poly, num_vars),
        };
        let claim = initial_state.map.iter().map(|(_, v)| v.iter().fold(F::ONE, F::mul)).fold(F::ZERO, F::add);
        return (claim, initial_state)
    }

    // We collapse the map based on the bit string prefix.
    pub fn round_phase_1(state: ProverState) -> (MVMLDescription, ProverState) {
        let (p0s, p1s) = collapse_map(&state.map);
        let polys: MVMLDescription = p0s.into_iter().zip(p1s.into_iter()).collect();
        (polys, state)
    }

    pub fn round_phase_2(state: ProverState, r: F) -> ProverState {
        let num_vars = state.num_vars - state.last_round - 1;
        let new_map = reduce_map(num_vars, r, &state.map);
        let new_state = ProverState {
            last_round: state.last_round + 1,
            map: new_map,
            ..state
        };
        new_state
    }
}

fn reduce_map(num_vars: usize, r: F, map: &HashMap<String, Vec<F>>) -> HashMap<String, Vec<F>> {
    (0..(2_u64.pow(num_vars as u32)))
        .map(|n|number_to_bit_string(n, num_vars))
        .map(|bit_string| (bit_string.clone(), combine_table_elements(bit_string, r, map)))
        .collect::<HashMap<String, Vec<F>>>()
}

fn combine_table_elements(bit_string: String, r: F, map: &HashMap<String, Vec<F>>) -> Vec<F> {
    let v0 = map.get(&format!("0{bit_string}")).unwrap();
    let v1 = map.get(&format!("1{bit_string}")).unwrap();
    return v0.iter().zip(v1.iter()).map(|(a0, a1)|a0.sub(&r.mul(a0)).add(&r.mul(a1))).collect()
}

fn collapse_map(map: &HashMap<String, Vec<F>>) -> (Vec<F>, Vec<F>) {
    map.iter()
        .fold((Vec::new(), Vec::new()), |acc, kv| add_to_acc(acc, kv))
}

fn add_to_acc((mut acc0, mut acc1): (Vec<F>, Vec<F>), (k, v): (&String, &Vec<F>)) -> (Vec<F>, Vec<F>){
    if acc0.is_empty() {
        acc0 = vec![F::ZERO; v.len()];
    }
    if acc1.is_empty() {
        acc1 = vec![F::ZERO; v.len()];
    }
    if k.starts_with("0") {
        (acc0.iter().zip(v.iter()).map(|(x, y)| x.add(y)).collect(), acc1)
    } else {
        (acc0, acc1.iter().zip(v.iter()).map(|(x, y)| x.add(y)).collect())
    }
}

#[cfg(test)]
mod tests {
    use ark_poly::DenseMVPolynomial;
    use ark_poly::multivariate::{SparsePolynomial, SparseTerm, Term};
    use crate::protocol::setup_protocol;
    use super::*;

    #[test]
    fn test_collapse_map() {
        let our_map = HashMap::from([
            ("000".to_owned(), [F::from(67)].to_vec()),
            ("001".to_owned(), [F::from(9)].to_vec()),
            ("010".to_owned(), [F::from(28)].to_vec()),
            ("011".to_owned(), [F::from(31)].to_vec()),
            ("100".to_owned(), [F::from(93)].to_vec()),
            ("101".to_owned(), [F::from(21)].to_vec()),
            ("110".to_owned(), [F::from(72)].to_vec()),
            ("111".to_owned(), [F::from(95)].to_vec()),
        ]);
        let p = collapse_map(&our_map);
        assert_eq!(p, (Vec::from([F::from(135)]), Vec::from([F::from(281)])));
    }

    #[test]
    fn test_collapse_map_product() {
        let our_map = HashMap::from([
            ("000".to_owned(), [F::from(67), F::from(76)].to_vec()),
            ("001".to_owned(), [F::from(9), F::from(91)].to_vec()),
            ("010".to_owned(), [F::from(28), F::from(82)].to_vec()),
            ("011".to_owned(), [F::from(31), F::from(13)].to_vec()),
            ("100".to_owned(), [F::from(93), F::from(39)].to_vec()),
            ("101".to_owned(), [F::from(21), F::from(12)].to_vec()),
            ("110".to_owned(), [F::from(72), F::from(27)].to_vec()),
            ("111".to_owned(), [F::from(95), F::from(36)].to_vec()),
        ]);
        let p = collapse_map(&our_map);
        assert_eq!(p, (Vec::from([F::from(135), F::from(262)]), Vec::from([F::from(281), F::from(114)])));
    }

    #[test]
    fn test_reduce_map() {

        let our_map = HashMap::from([
            ("000".to_owned(), [F::from(67)].to_vec()),
            ("001".to_owned(), [F::from(9)].to_vec()),
            ("010".to_owned(), [F::from(28)].to_vec()),
            ("011".to_owned(), [F::from(31)].to_vec()),
            ("100".to_owned(), [F::from(93)].to_vec()),
            ("101".to_owned(), [F::from(21)].to_vec()),
            ("110".to_owned(), [F::from(72)].to_vec()),
            ("111".to_owned(), [F::from(95)].to_vec()),
        ]);
        let r = F::from(83);
        let reduced = reduce_map(2, r, &our_map);
        let expected = HashMap::from([
            ("00".to_owned(), [F::from(2225)].to_vec()),
            ("01".to_owned(), [F::from(1005)].to_vec()),
            ("10".to_owned(), [F::from(3680)].to_vec()),
            ("11".to_owned(), [F::from(5343)].to_vec()),
        ]);

        assert!(reduced.eq(&expected));
    }

    #[test]
    fn test_reduce_map_product() {

        let our_map = HashMap::from([
            ("000".to_owned(), [F::from(67), F::from(76)].to_vec()),
            ("001".to_owned(), [F::from(9), F::from(91)].to_vec()),
            ("010".to_owned(), [F::from(28), F::from(82)].to_vec()),
            ("011".to_owned(), [F::from(31), F::from(13)].to_vec()),
            ("100".to_owned(), [F::from(93), F::from(39)].to_vec()),
            ("101".to_owned(), [F::from(21), F::from(12)].to_vec()),
            ("110".to_owned(), [F::from(72), F::from(27)].to_vec()),
            ("111".to_owned(), [F::from(95), F::from(36)].to_vec()),
        ]);
        let r = F::from(83);
        let reduced = reduce_map(2, r, &our_map);
        let expected = HashMap::from([
            ("00".to_owned(), [F::from(2225), F::from(-2995)].to_vec()),
            ("01".to_owned(), [F::from(1005), F::from(-6466)].to_vec()),
            ("10".to_owned(), [F::from(3680), F::from(-4483)].to_vec()),
            ("11".to_owned(), [F::from(5343), F::from(1922)].to_vec()),
        ]);

        assert!(reduced.eq(&expected));
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
        assert_eq!(Prover::claim_sum(&poly.unwrap()).0, F::from(93))
    }

    #[test]
    fn test_prover_phase_1() {
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
        let (poly_descr, prover_state) = Prover::round_phase_1(prover_state);
        let expected: MVMLDescription = Vec::from([(F::from(14), F::from(16)), (F::from(1), F::from(5))]);
        assert_eq!(poly_descr, expected)
    }
}