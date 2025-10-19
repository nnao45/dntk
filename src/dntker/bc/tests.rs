#[cfg(test)]
mod bc_tests {
    use crate::dntker::bc::BcExecuter;
    use dashu::base::Approximation;
    use dashu::Decimal;
    use rand::{RngCore, SeedableRng};

    fn expected_matrix_map<F>(exec: &mut BcExecuter, rows: &[Vec<&str>], mut map: F) -> String
    where
        F: FnMut(&str) -> String,
    {
        let mut formatted_rows = Vec::new();
        for row in rows {
            let mut values = Vec::new();
            for expr in row {
                let evaluation = map(expr);
                values.push(exec.exec(&evaluation).unwrap());
            }
            formatted_rows.push(format!("[{}]", values.join(", ")));
        }
        format!("[{}]", formatted_rows.join("; "))
    }

    fn expected_matrix(exec: &mut BcExecuter, rows: &[Vec<&str>]) -> String {
        expected_matrix_map(exec, rows, |expr| expr.to_string())
    }

    fn expected_sin_matrix(exec: &mut BcExecuter, rows: &[Vec<&str>]) -> String {
        expected_matrix_map(exec, rows, |expr| format!("sin({expr})"))
    }

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

    #[test]
    fn test_complex_expression_operations() {
        let mut exec: BcExecuter = Default::default();
        assert_eq!(exec.exec("3+2i").unwrap(), "3 + 2i");
        assert_eq!(exec.exec("2i").unwrap(), "2i");
        assert_eq!(exec.exec("(1+2i)*(3-4i)").unwrap(), "11 + 2i");
        assert_eq!(exec.exec("(4+2i)/(1-1i)").unwrap(), "1 + 3i");
        assert_eq!(exec.exec("abs(3+4i)").unwrap(), "5");
    }

    #[test]
    fn test_matrix_expression_operations() {
        let mut exec: BcExecuter = Default::default();
        assert_eq!(
            exec.exec("[[1,2],[3,4]] + [[5,6],[7,8]]").unwrap(),
            "[[6, 8]; [10, 12]]"
        );
        assert_eq!(
            exec.exec("[[5,5],[5,5]] - [[1,2],[3,4]]").unwrap(),
            "[[4, 3]; [2, 1]]"
        );
        assert_eq!(
            exec.exec("[[1,2],[3,4]] * [[5,6],[7,8]]").unwrap(),
            "[[19, 22]; [43, 50]]"
        );
        assert_eq!(exec.exec("2 * [[1,2],[3,4]]").unwrap(), "[[2, 4]; [6, 8]]");
        assert_eq!(exec.exec("[[2,4],[6,8]] / 2").unwrap(), "[[1, 2]; [3, 4]]");
    }

    #[test]
    fn test_matrix_sin_operations() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();

        let expected_0_to_3 = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(0)").unwrap(),
            exec.exec("sin(1)").unwrap(),
            exec.exec("sin(2)").unwrap(),
            exec.exec("sin(3)").unwrap()
        );
        assert_eq!(exec.exec("sin([[0,1],[2,3]])").unwrap(), expected_0_to_3);

        let expected_negatives = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(-1)").unwrap(),
            exec.exec("sin(-2)").unwrap(),
            exec.exec("sin(-3)").unwrap(),
            exec.exec("sin(-4)").unwrap()
        );
        assert_eq!(
            exec.exec("sin([[-1,-2],[-3,-4]])").unwrap(),
            expected_negatives
        );

        let expected_fractions = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(0.5)").unwrap(),
            exec.exec("sin(1.5)").unwrap(),
            exec.exec("sin(2.5)").unwrap(),
            exec.exec("sin(3.5)").unwrap()
        );
        assert_eq!(
            exec.exec("sin([[0.5,1.5],[2.5,3.5]])").unwrap(),
            expected_fractions
        );

        exec.exec("scale=4").unwrap();
        let expected_quarters = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(0)").unwrap(),
            exec.exec("sin(0.25)").unwrap(),
            exec.exec("sin(0.5)").unwrap(),
            exec.exec("sin(0.75)").unwrap()
        );
        assert_eq!(
            exec.exec("sin([[0,0.25],[0.5,0.75]])").unwrap(),
            expected_quarters
        );

        exec.exec("scale=6").unwrap();
        let expected_shifted = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(1)").unwrap(),
            exec.exec("sin(2)").unwrap(),
            exec.exec("sin(3)").unwrap(),
            exec.exec("sin(4)").unwrap()
        );
        assert_eq!(exec.exec("sin([[1,2],[3,4]])").unwrap(), expected_shifted);
    }

    #[test]
    fn test_complex_matrix_add_sub() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=6").unwrap();

        let complex_sum = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(1+2i)+(5-6i)").unwrap(),
            exec.exec("(3-4i)+(-3+4i)").unwrap(),
            exec.exec("(5i)+(-5i)").unwrap(),
            exec.exec("(-6+2i)+(6-2i)").unwrap()
        );
        assert_eq!(
            exec.exec("[[1+2i,3-4i],[5i,-6+2i]] + [[5-6i,-3+4i],[-5i,6-2i]]")
                .unwrap(),
            complex_sum
        );

        let complex_diff = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(7-8i)-(1+2i)").unwrap(),
            exec.exec("(3i)-(3i)").unwrap(),
            exec.exec("(2-2i)-(1+i)").unwrap(),
            exec.exec("(4+4i)-(2-2i)").unwrap()
        );
        assert_eq!(
            exec.exec("[[7-8i,3i],[2-2i,4+4i]] - [[1+2i,3i],[1+i,2-2i]]")
                .unwrap(),
            complex_diff
        );

        let mixed_sum = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(1+2i)+3").unwrap(),
            exec.exec("(4-5i)+6").unwrap(),
            exec.exec("(7+8i)+9").unwrap(),
            exec.exec("(10-11i)+12").unwrap()
        );
        assert_eq!(
            exec.exec("[[1+2i,4-5i],[7+8i,10-11i]] + [[3,6],[9,12]]")
                .unwrap(),
            mixed_sum
        );

        let real_minus_complex = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("3-(1+2i)").unwrap(),
            exec.exec("6-(4-5i)").unwrap(),
            exec.exec("9-(7+8i)").unwrap(),
            exec.exec("12-(10-11i)").unwrap()
        );
        assert_eq!(
            exec.exec("[[3,6],[9,12]] - [[1+2i,4-5i],[7+8i,10-11i]]")
                .unwrap(),
            real_minus_complex
        );

        let double_neg = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("-(1+2i)-(3-4i)").unwrap(),
            exec.exec("-(5i)-(-6+2i)").unwrap(),
            exec.exec("-(-7+8i)-(9-10i)").unwrap(),
            exec.exec("-(11i)-(-12i)").unwrap()
        );
        assert_eq!(
            exec.exec("-([[1+2i,5i],[-7+8i,11i]] + [[3-4i,-6+2i],[9-10i,-12i]])")
                .unwrap(),
            double_neg
        );
    }

    #[test]
    fn test_complex_matrix_multiplication() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=6").unwrap();

        let product = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(1+2i)*(2) + (3-4i)*(3+2i)").unwrap(),
            exec.exec("(1+2i)*(-(1i)) + (3-4i)*4").unwrap(),
            exec.exec("(5+6i)*(2) + (7-8i)*(3+2i)").unwrap(),
            exec.exec("(5+6i)*(-(1i)) + (7-8i)*4").unwrap()
        );
        assert_eq!(
            exec.exec("[[1+2i,3-4i],[5+6i,7-8i]] * [[2,-1i],[3+2i,4]]")
                .unwrap(),
            product
        );

        let real_mix = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(1+2i)*1 + (3+4i)*0").unwrap(),
            exec.exec("(1+2i)*2 + (3+4i)*1").unwrap(),
            exec.exec("(5-6i)*1 + (7-8i)*0").unwrap(),
            exec.exec("(5-6i)*2 + (7-8i)*1").unwrap()
        );
        assert_eq!(
            exec.exec("[[1+2i,3+4i],[5-6i,7-8i]] * [[1,2],[0,1]]")
                .unwrap(),
            real_mix
        );

        let scalar_left = exec.exec("(2+3i) * [[1+2i,3-4i],[5+6i,7-8i]]").unwrap();
        let scalar_left_expected = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(2+3i)*(1+2i)").unwrap(),
            exec.exec("(2+3i)*(3-4i)").unwrap(),
            exec.exec("(2+3i)*(5+6i)").unwrap(),
            exec.exec("(2+3i)*(7-8i)").unwrap()
        );
        assert_eq!(scalar_left, scalar_left_expected);

        let scalar_right = exec.exec("[[1+2i,3-4i],[5+6i,7-8i]] * (2+3i)").unwrap();
        assert_eq!(scalar_right, scalar_left_expected);

        let identity = exec
            .exec("[[1+2i,3-4i],[5+6i,7-8i]] * [[1,0],[0,1]]")
            .unwrap();
        let original = exec.exec("[[1+2i,3-4i],[5+6i,7-8i]]").unwrap();
        assert_eq!(identity, original);
    }

    #[test]
    fn test_complex_matrix_scalar_division() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=6").unwrap();

        exec.exec("[[1+2i,3-4i],[5+6i,7-8i]] * (3-2i)").unwrap();
        let divided = exec
            .exec("([[1+2i,3-4i],[5+6i,7-8i]] * (3-2i)) / (3-2i)")
            .unwrap();
        let original = exec.exec("[[1+2i,3-4i],[5+6i,7-8i]]").unwrap();
        assert_eq!(divided, original);

        let divide_complex = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(1+2i)/(2+3i)").unwrap(),
            exec.exec("(3-4i)/(2+3i)").unwrap(),
            exec.exec("(5+6i)/(2+3i)").unwrap(),
            exec.exec("(7-8i)/(2+3i)").unwrap()
        );
        assert_eq!(
            exec.exec("[[1+2i,3-4i],[5+6i,7-8i]] / (2+3i)").unwrap(),
            divide_complex
        );

        let divide_real = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("(1+2i)/2").unwrap(),
            exec.exec("(3-4i)/2").unwrap(),
            exec.exec("(5+6i)/2").unwrap(),
            exec.exec("(7-8i)/2").unwrap()
        );
        assert_eq!(
            exec.exec("[[1+2i,3-4i],[5+6i,7-8i]] / 2").unwrap(),
            divide_real
        );

        let combine = exec
            .exec("(([[1+2i,3-4i],[5+6i,7-8i]] + [[-1+2i,4-3i],[2i,-2i]]) * (1+1i)) / (1+1i)")
            .unwrap();
        let sum = exec
            .exec("[[1+2i,3-4i],[5+6i,7-8i]] + [[-1+2i,4-3i],[2i,-2i]]")
            .unwrap();
        assert_eq!(combine, sum);

        let symmetric_diff = exec
            .exec("(([[1+2i,3-4i],[5+6i,7-8i]] / (2-3i)) * (2-3i)) - [[1+2i,3-4i],[5+6i,7-8i]]")
            .unwrap();
        let normalized_diff = symmetric_diff.replace(" + 0i", "").replace(" - 0i", "");
        assert_eq!(normalized_diff, "[[0, 0]; [0, 0]]");
    }

    #[test]
    fn test_complex_matrix_sin() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=6").unwrap();

        let expected = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(1+i)").unwrap(),
            exec.exec("sin(-1+i)").unwrap(),
            exec.exec("sin(2-2i)").unwrap(),
            exec.exec("sin(-2-2i)").unwrap()
        );
        assert_eq!(
            exec.exec("sin([[1+i,-1+i],[2-2i,-2-2i]])").unwrap(),
            expected
        );

        let expected_mixed = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(0)").unwrap(),
            exec.exec("sin(1+i)").unwrap(),
            exec.exec("sin(2)").unwrap(),
            exec.exec("sin(-3i)").unwrap()
        );
        assert_eq!(exec.exec("sin([[0,1+i],[2,-3i]])").unwrap(), expected_mixed);

        exec.exec("scale=4").unwrap();
        let expected_scale = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin(0.5+0.5i)").unwrap(),
            exec.exec("sin(1.5-0.5i)").unwrap(),
            exec.exec("sin(-0.5+1.5i)").unwrap(),
            exec.exec("sin(-1.5-1.5i)").unwrap()
        );
        assert_eq!(
            exec.exec("sin([[0.5+0.5i,1.5-0.5i],[-0.5+1.5i,-1.5-1.5i]])")
                .unwrap(),
            expected_scale
        );

        exec.exec("scale=6").unwrap();
        let expected_combo = format!(
            "[[{}, {}]; [{}, {}]]",
            exec.exec("sin((1+i)+(2-2i))").unwrap(),
            exec.exec("sin((3-3i)-(1+i))").unwrap(),
            exec.exec("sin((2i)*(1+i))").unwrap(),
            exec.exec("sin((-2i)*(1-i))").unwrap()
        );
        assert_eq!(
            exec.exec("sin([[(1+i)+(2-2i),(3-3i)-(1+i)],[(2i)*(1+i),(-2i)*(1-i)]])",)
                .unwrap(),
            expected_combo
        );

        let expected_identity = exec
            .exec("[[sin(0), sin(1)]; [sin(2), sin(3)]]")
            .unwrap_or_else(|_| {
                let a = exec.exec("sin(0)").unwrap();
                let b = exec.exec("sin(1)").unwrap();
                let c = exec.exec("sin(2)").unwrap();
                let d = exec.exec("sin(3)").unwrap();
                format!("[[{}, {}]; [{}, {}]]", a, b, c, d)
            });
        assert_eq!(exec.exec("sin([[0,1],[2,3]])").unwrap(), expected_identity);
    }

    #[test]
    fn test_matrix_sin_zero_matrix() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=10").unwrap();
        let pattern = vec![vec!["0", "0"], vec!["0", "0"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[0,0],[0,0]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_large_angles() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![vec!["10", "20"], vec!["-10", "-20"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[10,20],[-10,-20]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_fractional_rows() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=12").unwrap();
        let pattern = vec![vec!["0.333333333333", "0.666666666667"], vec!["0.6", "0.8"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec
            .exec("sin([[0.333333333333,0.666666666667],[0.6,0.8]])")
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_scaled_precision_variation() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=18").unwrap();
        let pattern = vec![
            vec!["0.123456789", "1.23456789"],
            vec!["2.3456789", "3.456789"],
        ];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec
            .exec("sin([[0.123456789,1.23456789],[2.3456789,3.456789]])")
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_with_add_sub_mix() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=9").unwrap();
        let pattern = vec![vec!["3", "2"], vec!["2", "-2.5"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[3,2],[2,-2.5]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_scalar_multiplication_argument() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![vec!["3*1", "3*2"], vec!["3*3", "3*4"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin(3*[[1,2],[3,4]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_rectangular_two_by_three() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![vec!["0", "1", "2"], vec!["3", "4", "5"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[0,1,2],[3,4,5]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_rectangular_three_by_one() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![vec!["0"], vec!["1"], vec!["2"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[0],[1],[2]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_nested_call() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=10").unwrap();
        let pattern = vec![vec!["1", "2"], vec!["3", "4"]];
        let expected = expected_matrix_map(&mut exec, &pattern, |expr| format!("sin(sin({expr}))"));
        let actual = exec.exec("sin(sin([[1,2],[3,4]]))").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_matrix_sin_after_scale_change() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=4").unwrap();
        exec.exec("sin([[1,2],[3,4]])").unwrap();
        exec.exec("scale=9").unwrap();
        let pattern = vec![
            vec!["0.714285714", "0.857142857"],
            vec!["0.875", "0.888888889"],
        ];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec
            .exec("sin([[0.714285714,0.857142857],[0.875,0.888888889]])")
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_complex_matrix_sin_with_zero_imaginary() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=10").unwrap();
        let pattern = vec![vec!["1+0i", "2+0i"], vec!["-3+0i", "4+0i"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[1+0i,2+0i],[-3+0i,4+0i]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_complex_matrix_sin_scalar_commutation() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=6").unwrap();
        let left = exec.exec("sin((2+3i) * [[1+i,2-1i],[3+2i,4-2i]])").unwrap();
        let right = exec.exec("sin([[1+i,2-1i],[3+2i,4-2i]] * (2+3i))").unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn test_complex_matrix_sin_rectangular() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![vec!["1+i", "-1i", "2i"]];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec.exec("sin([[1+i,-1i,2i]])").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_complex_matrix_sin_negation_symmetry() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=6").unwrap();
        let base = exec.exec("sin([[1+i,-2i],[3-3i,-4+4i]])").unwrap();
        let negated = exec.exec("sin(-[[1+i,-2i],[3-3i,-4+4i]])").unwrap();
        let expected_neg = exec.exec(&format!("-({base})")).unwrap();
        assert_eq!(negated, expected_neg);
    }

    #[test]
    fn test_complex_matrix_sin_mixed_operations() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![
            vec!["((1+i)+(-1i))/2", "((2-3i)+(3i))/2"],
            vec!["((4+5i)+(2-1i))/2", "((6-7i)+(-4+4i))/2"],
        ];
        let expected = expected_sin_matrix(&mut exec, &pattern);
        let actual = exec
            .exec("sin(([[1+i,2-3i],[4+5i,6-7i]] + [[-1i,3i],[2-1i,-4+4i]]) / 2)")
            .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_complex_matrix_add_commutes() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=7").unwrap();
        let ab = exec
            .exec("[[1+i,2-3i],[4+5i,6-7i]] + [[-1i,3],[5i,-2i]]")
            .unwrap();
        let ba = exec
            .exec("[[-1i,3],[5i,-2i]] + [[1+i,2-3i],[4+5i,6-7i]]")
            .unwrap();
        assert_eq!(ab, ba);
    }

    #[test]
    fn test_complex_matrix_scalar_distribution() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let left = exec
            .exec("(2+3i) * ([[1+i,2-1i],[3+4i,5-6i]] + [[-1+2i,3-3i],[4+0i,-5i]])")
            .unwrap();
        let right = exec
            .exec("(2+3i)*[[1+i,2-1i],[3+4i,5-6i]] + (2+3i)*[[-1+2i,3-3i],[4+0i,-5i]]")
            .unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn test_complex_matrix_division_matches_inverse_multiplication() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let pattern = vec![
            vec!["(1+i)/(1+2i)", "(2-3i)/(1+2i)"],
            vec!["(3+4i)/(1+2i)", "(5-6i)/(1+2i)"],
        ];
        let expected = expected_matrix(&mut exec, &pattern);
        let actual = exec.exec("[[1+i,2-3i],[3+4i,5-6i]] / (1+2i)").unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_complex_matrix_negation_matches_scalar() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=8").unwrap();
        let negated = exec.exec("-[[1+i,2-2i],[3+3i,-4i]]").unwrap();
        let scaled = exec.exec("(-1)*[[1+i,2-2i],[3+3i,-4i]]").unwrap();
        assert_eq!(negated, scaled);
    }

    #[test]
    fn test_complex_matrix_affine_combination() {
        let mut exec: BcExecuter = Default::default();
        exec.exec("scale=10").unwrap();
        let pattern = vec![
            vec!["((1+i)+(2-3i))/2 - (3i)/4", "((4+5i)+(-6+7i))/2 - (-2i)/4"],
            vec![
                "((5-5i)+(6+6i))/2 - (1-2i)/4",
                "((7+0i)+(-8+1i))/2 - (-3+4i)/4",
            ],
        ];
        let expected = expected_matrix(&mut exec, &pattern);
        let actual = exec
            .exec(
                "(1/2)*([[1+i,4+5i],[5-5i,7+0i]] + [[2-3i,-6+7i],[6+6i,-8+1i]]) - (1/4)*[[3i,-2i],[1-2i,-3+4i]]",
            )
            .unwrap();
        assert_eq!(actual, expected);
    }
}
