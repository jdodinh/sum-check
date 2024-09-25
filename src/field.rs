use ark_ff::{
    fields::{MontConfig, Fp256, MontBackend},
};

#[derive(MontConfig)]
#[modulus="57896044618658097711785492504343953926634992332820282019728792003956564819949"]
#[generator="2"]
pub struct FieldConfig;

pub type Field256 = Fp256<MontBackend<FieldConfig, 4>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let el_1 = Field256::from(3);
        let el_2 = Field256::from(6);
        assert_eq!(el_1 + el_2, Field256::from(9));
        assert_eq!(el_1 + el_2 + el_2, Field256::from(15));
    }

    #[test]
    fn test_subtraction() {
        let el_1 = Field256::from(3);
        let el_2 = Field256::from(6);
        assert_eq!(el_1 - el_2, Field256::from(-3));
    }
}
