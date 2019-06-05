#[cfg(test)]
mod bc_tests {
    use super::{BcExecuter};

    #[test]
    fn test_handle_output(){
        let b = BcExecuter::new();
        let input1 = "1";
        let input1_newline = format!("{}{}", input1, "\n");
        assert_eq!(b.handle_output(input1_newline.to_string()), input1.to_string());
        let input2 = "0.12";
        let input2_newline = format!("{}{}", input2, "\n");
        assert_eq!(b.handle_output(input2_newline.to_string()), input2.to_string());
        let input3 = "211";
        let input3_newline = format!("{}{}", input3, "\\\n\n");
        assert_eq!(b.handle_output(input3_newline.to_string()), input3.to_string());
    }

    #[test]
    fn test_exec(){
        let b = BcExecuter::new();
        let input1 = "1+2";
        let output1= "3";
        assert_eq!(b.exec(input1).unwrap(), output1);
        let input2 = "0.12/a(123)";
        let output2 = ".07679182076851013335";
        assert_eq!(b.exec(input2).unwrap(), output2);
        let input3 = "2^2^2^2";
        let output3 = "65536";
        assert_eq!(b.exec(input3).unwrap(), output3);
        let input4 = "3x4x";
        #[cfg(not(target_os = "macos"))]
        let output4 = "Error(\"(standard_in) 1: parse error\")";
        #[cfg(target_os = "macos")]
        let output4 = "Error(\"(standard_in) 1: parse error\")";
        assert_eq!(format!("{:?}", b.exec(input4).err().unwrap()), output4);
    }
}