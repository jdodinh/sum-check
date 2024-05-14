use ark_ff::{
    fields::{MontConfig, Fp64, MontBackend},
};

#[derive(MontConfig)]
#[modulus="18446744069414584321"]
#[generator="2"]
pub struct FieldConfig;

pub type Field64 = Fp64<MontBackend<FieldConfig, 1>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let el_1 = Field64::from(3);
        let el_2 = Field64::from(6);
        assert_eq!(el_1 + el_2, Field64::from(9));
        assert_eq!(el_1 + el_2 + el_2, Field64::from(15));
    }

    #[test]
    fn test_subtraction() {
        let el_1 = Field64::from(3);
        let el_2 = Field64::from(6);
        assert_eq!(el_1 - el_2, Field64::from(18446744069414584318u128));
    }
}