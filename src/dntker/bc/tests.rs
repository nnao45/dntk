#[cfg(test)]
mod bc_tests {
    use super::{BcExecuter};

    #[test]
    fn test_exec(){
        let mut b: BcExecuter = Default::default();

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

    #[test]
    fn test_exec_with_assignments_and_multi_statements() {
        let mut b: BcExecuter = Default::default();
        let script = "a=1+2; b=a*4; a+b";
        let output = b.exec(script).unwrap();
        assert_eq!(output, "15");
    }

    #[test]
    fn test_exec_with_for_loop() {
        let mut b: BcExecuter = Default::default();
        let script = "sum=0; for(i=1; i<=3; i=i+1){ sum = sum + i; }; sum";
        let output = b.exec(script).unwrap();
        assert_eq!(output, "6");
    }
}
