#[cfg(test)]
mod bc_tests {
    use super::{BcExecuter};

    #[test]
    fn test_exec(){
        let b: BcExecuter = Default::default();

        // Basic arithmetic
        let input1 = "1+2";
        let output1= "3";
        assert_eq!(b.exec(input1).unwrap(), output1);

        // Division with atan function (bc: a() = atan())
        let input2 = "0.12/atan(123)";
        let result2 = b.exec(input2).unwrap();
        // Check that the result is approximately correct
        let expected_approx = 0.07679182076851013335;
        let actual: f64 = result2.parse().unwrap();
        assert!((actual - expected_approx).abs() < 1e-10,
                "Expected ~{}, got {}", expected_approx, actual);

        // Exponentiation
        let input3 = "2^2^2^2";
        let output3 = "65536";
        assert_eq!(b.exec(input3).unwrap(), output3);
    }
}