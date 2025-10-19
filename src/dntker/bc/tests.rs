#[cfg(test)]
mod bc_tests {
    use crate::dntker::bc::BcExecuter;
    use dashu::base::Approximation;
    use dashu::Decimal;
    use rand::{RngCore, SeedableRng};

    fn with_precision(value: Decimal, precision: usize) -> Decimal {
        match value.with_precision(precision) {
            Approximation::Exact(v) | Approximation::Inexact(v, _) => v,
        }
    }

    #[test]
    fn test_exec() {
        let mut b: BcExecuter = Default::default();

        // Basic arithmetic
        let input1 = "1+2";
        let output1 = "3";
        assert_eq!(b.exec(input1).unwrap(), output1);

        // Division with atan function (bc: a() = atan())
        let input2 = "0.12/atan(123)";
        let result2 = b.exec(input2).unwrap();
        // Check that the result is approximately correct
        let expected_approx = 0.07679182076851013335;
        let actual: f64 = result2.parse().unwrap();
        assert!(
            (actual - expected_approx).abs() < 1e-10,
            "Expected ~{}, got {}",
            expected_approx,
            actual
        );

        // Exponentiation
        let input3 = "2^2^2^2";
        let output3 = "65536";
        assert_eq!(b.exec(input3).unwrap(), output3);
    }

    #[test]
    fn test_high_precision_division_matches_dashu() {
        let mut exec: BcExecuter = Default::default();
        let output = exec.exec("scale=50; 2/3").unwrap();
        let decimal_result: Decimal = output.parse().unwrap();

        let numerator = with_precision(Decimal::from(2), 200);
        let denominator = with_precision(Decimal::from(3), 200);
        let expected = numerator / denominator;
        let expected = BcExecuter::truncate_decimal_to_scale(&expected, 50);

        assert_eq!(decimal_result, expected);
        let expected_string = exec.format_result_decimal(&expected);
        assert_eq!(output, expected_string);
    }

    #[test]
    fn test_variable_preserves_high_precision() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=40; a=10/3").unwrap();
        let output = exec.exec("a").unwrap();
        let decimal_result: Decimal = output.parse().unwrap();

        let numerator = with_precision(Decimal::from(10), 200);
        let denominator = with_precision(Decimal::from(3), 200);
        let expected = numerator / denominator;
        let expected = BcExecuter::truncate_decimal_to_scale(&expected, 40);
        assert_eq!(decimal_result, expected);
        assert_eq!(output, exec.format_result_decimal(&expected));
    }

    #[test]
    fn test_scale_zero_truncates_results() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=0").unwrap();
        assert_eq!(exec.exec("2/3").unwrap(), "0");
        assert_eq!(exec.exec("-2/3").unwrap(), "0");
        assert_eq!(exec.exec("5/2").unwrap(), "2");
    }

    #[test]
    fn test_scale_padding_preserves_trailing_zeros() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=5").unwrap();
        assert_eq!(exec.exec("1/2").unwrap(), ".50000");
        assert_eq!(exec.exec("1/5").unwrap(), ".20000");
        assert_eq!(exec.exec("10/5").unwrap(), "2");
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

    #[test]
    fn test_exec_with_if_else() {
        let mut b: BcExecuter = Default::default();
        let script = "a=5; if(a>3){ b=10; } else { b=1; }; b";
        let output = b.exec(script).unwrap();
        assert_eq!(output, "10");
    }

    #[test]
    fn test_exec_with_while() {
        let mut b: BcExecuter = Default::default();
        let script = "i=0; total=0; while(i<4){ total=total+i; i=i+1; }; total";
        let output = b.exec(script).unwrap();
        assert_eq!(output, "6");
    }

    #[test]
    fn test_exec_with_function_call() {
        let mut b: BcExecuter = Default::default();
        b.exec("define add_twice(x){ return x + x; }").unwrap();
        b.exec("define mix(a,b){ tmp = add_twice(a); tmp + b; }")
            .unwrap();
        let output = b.exec("mix(3,4)").unwrap();
        assert_eq!(output, "10");
    }

    #[test]
    fn test_exec_with_additional_builtins() {
        let mut b: BcExecuter = Default::default();
        assert_eq!(b.exec("length(123456)").unwrap(), "6");
        assert_eq!(b.exec("scale(12.345)").unwrap(), "3");
        let bessel = b.exec("j(1, 2)");
        assert!(bessel.is_ok());
        let bessel_val: f64 = bessel.unwrap().parse().unwrap();
        assert!((bessel_val - 0.5767248077568734).abs() < 1e-9);

        assert_eq!(b.exec("sqrt(9)").unwrap(), "3");
        assert_eq!(b.exec("abs(-5)").unwrap(), "5");
        assert_eq!(b.exec("sign(-3)").unwrap(), "-1");
        assert_eq!(b.exec("cbrt(27)").unwrap(), "3");
        assert_eq!(b.exec("round(2.3)").unwrap(), "2");
        assert_eq!(b.exec("floor(2.9)").unwrap(), "2");
        assert_eq!(b.exec("ceil(2.1)").unwrap(), "3");
        assert_eq!(b.exec("sin(0)").unwrap(), "0");
        assert_eq!(b.exec("cos(0)").unwrap(), "1");
        assert_eq!(b.exec("log(100)").unwrap(), "2");
        assert_eq!(b.exec("log(2,8)").unwrap(), "3");
        assert_eq!(b.exec("log10(1000)").unwrap(), "3");
        assert_eq!(b.exec("log2(8)").unwrap(), "3");
        assert_eq!(b.exec("pow(2,3)").unwrap(), "8");
        assert_eq!(b.exec("hypot(3,4)").unwrap(), "5");
        assert_eq!(b.exec("min(-2, 4)").unwrap(), "-2");
        assert_eq!(b.exec("max(-2, 4, 1)").unwrap(), "4");

        assert_eq!(b.exec("scale=5; 1/3").unwrap(), ".33333");
        assert_eq!(b.exec("scale").unwrap(), "5");

        assert_eq!(b.exec("obase=16; 255").unwrap(), "FF");
        assert_eq!(b.exec("obase").unwrap(), "10");
        assert_eq!(b.exec("obase=10; sqrt(16)").unwrap(), "4");

        let rand_seed_output = b.exec("srand(42)").unwrap();
        assert_eq!(rand_seed_output, "42");
        let rand_value: f64 = b.exec("rand()").unwrap().parse().unwrap();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let expected = (rng.next_u32() & 0x7fff) as f64;
        assert_eq!(rand_value, expected);
        let bounded: f64 = b.exec("rand(5)").unwrap().parse().unwrap();
        assert!(bounded >= 0.0 && bounded < 5.0);
    }
}
