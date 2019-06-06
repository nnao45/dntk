#[cfg(test)]
mod dntker_tests {
    use super::{Dntker, util, FilterResult, DntkResult, bc, DntkString, DntkStringType};
    #[test]
    fn test_filter_char() {
        let d = Dntker::new();
        assert_eq!(d.filter_char(util::ASCII_CODE_ZERO       ), FilterResult::BcCode(util::ASCII_CODE_ZERO      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ONE        ), FilterResult::BcCode(util::ASCII_CODE_ONE       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_TWO        ), FilterResult::BcCode(util::ASCII_CODE_TWO       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_THREE      ), FilterResult::BcCode(util::ASCII_CODE_THREE     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_FOUR       ), FilterResult::BcCode(util::ASCII_CODE_FOUR      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_FIVE       ), FilterResult::BcCode(util::ASCII_CODE_FIVE      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SIX        ), FilterResult::BcCode(util::ASCII_CODE_SIX       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SEVEN      ), FilterResult::BcCode(util::ASCII_CODE_SEVEN     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_EIGHT      ), FilterResult::BcCode(util::ASCII_CODE_EIGHT     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_NINE       ), FilterResult::BcCode(util::ASCII_CODE_NINE      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_S          ), FilterResult::BcCode(util::ASCII_CODE_S         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_C          ), FilterResult::BcCode(util::ASCII_CODE_C         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_A          ), FilterResult::BcCode(util::ASCII_CODE_A         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_L          ), FilterResult::BcCode(util::ASCII_CODE_L         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_E          ), FilterResult::BcCode(util::ASCII_CODE_E         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_J          ), FilterResult::BcCode(util::ASCII_CODE_J         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_R          ), FilterResult::BcCode(util::ASCII_CODE_R         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_Q          ), FilterResult::BcCode(util::ASCII_CODE_Q         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_T          ), FilterResult::BcCode(util::ASCII_CODE_T         ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ROUNDLEFT  ), FilterResult::BcCode(util::ASCII_CODE_ROUNDLEFT ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ROUNDRIGHT ), FilterResult::BcCode(util::ASCII_CODE_ROUNDRIGHT));
        assert_eq!(d.filter_char(util::ASCII_CODE_LARGER     ), FilterResult::BcCode(util::ASCII_CODE_LARGER    ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SMALLER    ), FilterResult::BcCode(util::ASCII_CODE_SMALLER   ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SQUARELEFT ), FilterResult::CurLeftCode                        );
        assert_eq!(d.filter_char(util::ASCII_CODE_SQUARERIGHT), FilterResult::CurRightCode                       );
        assert_eq!(d.filter_char(util::ASCII_CODE_PLUS       ), FilterResult::BcCode(util::ASCII_CODE_PLUS      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_MINUS      ), FilterResult::BcCode(util::ASCII_CODE_MINUS     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_ASTERISK   ), FilterResult::BcCode(util::ASCII_CODE_ASTERISK  ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SLUSH      ), FilterResult::BcCode(util::ASCII_CODE_SLUSH     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_HAT        ), FilterResult::BcCode(util::ASCII_CODE_HAT       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_PERCENT    ), FilterResult::BcCode(util::ASCII_CODE_PERCENT   ));
        assert_eq!(d.filter_char(util::ASCII_CODE_DOT        ), FilterResult::BcCode(util::ASCII_CODE_DOT       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_BIKKURI    ), FilterResult::BcCode(util::ASCII_CODE_BIKKURI   ));
        assert_eq!(d.filter_char(util::ASCII_CODE_EQUAL      ), FilterResult::BcCode(util::ASCII_CODE_EQUAL     ));
        assert_eq!(d.filter_char(util::ASCII_CODE_PIPE       ), FilterResult::BcCode(util::ASCII_CODE_PIPE      ));
        assert_eq!(d.filter_char(util::ASCII_CODE_AND        ), FilterResult::BcCode(util::ASCII_CODE_AND       ));
        assert_eq!(d.filter_char(util::ASCII_CODE_SEMICOLON  ), FilterResult::BcCode(util::ASCII_CODE_SEMICOLON ));
        assert_eq!(d.filter_char(util::ASCII_CODE_AT         ), FilterResult::RefreshCode                        );
        assert_eq!(d.filter_char(util::ASCII_CODE_WINENTER   ), FilterResult::EndCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_NEWLINE    ), FilterResult::EndCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_ESCAPE     ), FilterResult::EscCode                            );
        assert_eq!(d.filter_char(util::ASCII_CODE_BACKSPACE  ), FilterResult::DeleteCode                         );
        assert_eq!(d.filter_char(util::ASCII_CODE_DELETE     ), FilterResult::DeleteCode                         );
        assert_eq!(d.filter_char(util::ASCII_CODE_SPACE      ), FilterResult::BcCode(util::ASCII_CODE_SPACE     ));

        assert_eq!(d.filter_char(0x00                        ), FilterResult::UnknownCode(0x00                  ));
        assert_eq!(d.filter_char(0x0e                        ), FilterResult::UnknownCode(0x0e                  ));
        assert_eq!(d.filter_char(0x4f                        ), FilterResult::UnknownCode(0x4f                  ));
    }

    #[test]
    fn test_delete_column() {
        let d1 = &mut Dntker::new();
        d1.delete_column();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.delete_column();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, test_currnet_cur_pos-1);

        assert_eq!(d2.before_printed_len, test_before_printed_len);
        assert_eq!(d2.before_printed_statement_len, test_before_printed_statement_len);
    }

    #[test]
    fn test_cursor_move_left() {
        let d1 = &mut Dntker::new();
        d1.cursor_move_left();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_result_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.cursor_move_left();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, 1);

        assert_eq!(d2.before_printed_len, 3);
        assert_eq!(d2.before_printed_result_len, 1);
        assert_eq!(d2.before_printed_statement_len, 3);
    }

    #[test]
    fn test_cursor_move_right() {
        let d1 = &mut Dntker::new();
        d1.cursor_move_right();

        assert_eq!(d1.input_vec, vec![]);
        assert_eq!(d1.currnet_cur_pos, 0);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_result_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.cursor_move_right();

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, 3);

        assert_eq!(d2.before_printed_len, 3);
        assert_eq!(d2.before_printed_result_len, 1);
        assert_eq!(d2.before_printed_statement_len, 3);
    }

    #[test]
    fn test_insert_column(){
        let d1 = &mut Dntker::new();
        let test_item = util::ASCII_CODE_THREE;
        d1.insert_column(&test_item);

        assert_eq!(d1.input_vec, vec![util::ASCII_CODE_THREE]);
        assert_eq!(d1.currnet_cur_pos, 1);

        assert_eq!(d1.before_printed_len, 0);
        assert_eq!(d1.before_printed_result_len, 0);
        assert_eq!(d1.before_printed_statement_len, 0);

        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d2.insert_column(&test_item);

        assert_eq!(d2.input_vec, vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_THREE, util::ASCII_CODE_TWO]);
        assert_eq!(d2.currnet_cur_pos, 3);

        assert_eq!(d2.before_printed_len, 3);
        assert_eq!(d2.before_printed_result_len, 1);
        assert_eq!(d2.before_printed_statement_len, 3);
    }

    #[test]
    fn test_statement_from_utf8() {
        let test_input_vec1 = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_input_vec2 = vec![util::ASCII_CODE_S, util::ASCII_CODE_ROUNDLEFT, util::ASCII_CODE_EIGHT, util::ASCII_CODE_ROUNDRIGHT];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d1 = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec1,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        let d2 = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec2,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        assert_eq!("1+2".to_string(), d1.statement_from_utf8());
        assert_eq!("s(8)".to_string(), d2.statement_from_utf8());
    }

    #[test]
    fn test_output_fill_whitespace() {
        let d = Dntker::new();
        assert_eq!("\r".to_string(), d.output_fill_whitespace(0));
        assert_eq!("\r ".to_string(), d.output_fill_whitespace(1));
        assert_eq!("\r    ".to_string(), d.output_fill_whitespace(4));
    }

    #[test]
    fn test_output_ok() {
        let d = &mut Dntker::new();
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m\u{1b}[7D".to_string(), d.output_ok(util::DNTK_PROMPT, "1+2", " = ", "3").ancize().to_string());
            assert_eq!("\u{1b}[36m\r(dntk): a(123) = 1.56266642461495270762\u{1b}[0m\u{1b}[31D".to_string(), d.output_ok(util::DNTK_PROMPT, "a(123)", " = ", "1.56266642461495270762").ancize().to_string());
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), d.output_ok(util::DNTK_PROMPT, "1+2", " = ", "3").ancize().to_string());
            assert_eq!("\u{1b}[36m\r(dntk): a(123) = 1.56266642461495270762\u{1b}[0m".to_string(), d.output_ok(util::DNTK_PROMPT, "a(123)", " = ", "1.56266642461495270762").ancize().to_string());
        }
    }

    #[test]
    fn test_output_ng() {
        let d = &mut Dntker::new();
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!("\u{1b}[35m\r(dntk): 1+2* = \u{1b}[0m\u{1b}[7D".to_string(), d.output_ng(util::DNTK_PROMPT, "1+2*", " = ").ancize().to_string());
            assert_eq!("\u{1b}[35m\r(dntk): a(123)*s( = \u{1b}[0m\u{1b}[12D".to_string(), d.output_ng(util::DNTK_PROMPT, "a(123)*s(", " = ").ancize().to_string());
        }
        #[cfg(target_os = "windows")]
        {
            assert_eq!("\u{1b}[35m\r(dntk): 1+2* = \u{1b}[0m".to_string(), d.output_ng(util::DNTK_PROMPT, "1+2*", " = ").ancize().to_string());
            assert_eq!("\u{1b}[35m\r(dntk): a(123)*s( = \u{1b}[0m".to_string(), d.output_ng(util::DNTK_PROMPT, "a(123)*s(", " = ").ancize().to_string());
        }
    }

    #[test]
    fn test_refresh() {
        let test_input_vec = vec![util::ASCII_CODE_ONE , util::ASCII_CODE_PLUS, util::ASCII_CODE_TWO];
        let test_before_printed_len = 3;
        let test_before_printed_result_len = 1;
        let test_before_printed_statement_len = 3;
        let test_currnet_cur_pos = 2;
        let d = &mut Dntker {
            executer: bc::BcExecuter::new(),
            input_vec: test_input_vec,
            before_printed_len: test_before_printed_len,
            before_printed_result_len: test_before_printed_result_len,
            before_printed_statement_len: test_before_printed_statement_len,
            currnet_cur_pos: test_currnet_cur_pos,
        };
        d.refresh();
        assert_eq!(d, &mut Dntker::new());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_dntk_exec() {
        let d1 = &mut Dntker::new();
        let ptr_escape: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as i8; 3];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_escape));
        let ptr1: [libc::c_char; 3] = [util::ASCII_CODE_ONE as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m".to_string()), d1.dntk_exec(ptr1));
        let ptr_right: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as i8, 0x91 as u8 as i8, util::ASCII_CODE_RIGHT as i8];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m".to_string()), d1.dntk_exec(ptr_right));
        let ptr2: [libc::c_char; 3] = [util::ASCII_CODE_PLUS as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[35m\r(dntk): 1+ = \u{1b}[0m".to_string()), d1.dntk_exec(ptr2));
        let ptr3: [libc::c_char; 3] = [util::ASCII_CODE_ZERO as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0 = 1\u{1b}[0m".to_string()), d1.dntk_exec(ptr3));
        let ptr4: [libc::c_char; 3] = [util::ASCII_CODE_DOT as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m".to_string()), d1.dntk_exec(ptr4));
        let ptr_unknown_ascii: [libc::c_char; 3] = [0x4f as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m".to_string()), d1.dntk_exec(ptr_unknown_ascii));
        let ptr5: [libc::c_char; 3] = [util::ASCII_CODE_SEVEN as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0.7 = 1.7\u{1b}[0m".to_string()), d1.dntk_exec(ptr5));
        let ptr_enter: [libc::c_char; 3] = [util::ASCII_CODE_NEWLINE as i8; 3];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_enter));
    }

    // #[test]
    // #[cfg(target_arch = "aarch64")]
    // fn test_dntk_exec() {
    //     let d1 = &mut Dntker::new();
    //     let ptr_escape: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as u8; 3];
    //     assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_escape));
    //     let ptr1: [libc::c_char; 3] = [util::ASCII_CODE_ONE as u8; 3];
    //     assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr1));
    //     let ptr_right: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as u8, 0x91 as u8, util::ASCII_CODE_RIGHT as u8];
    //     assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_right));
    //     let ptr2: [libc::c_char; 3] = [util::ASCII_CODE_PLUS as u8; 3];
    //     assert_eq!(DntkResult::Output("\u{1b}[35m\r(dntk): 1+ = \u{1b}[0m\u{1b}[3D".to_string()), d1.dntk_exec(ptr2));
    //     let ptr3: [libc::c_char; 3] = [util::ASCII_CODE_ZERO as u8; 3];
    //     assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr3));
    //     let ptr4: [libc::c_char; 3] = [util::ASCII_CODE_DOT as u8; 3];
    //     assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr4));
    //     let ptr_unknown_ascii: [libc::c_char; 3] = [0x4f as u8; 3];
    //     assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_unknown_ascii));
    //     let ptr5: [libc::c_char; 3] = [util::ASCII_CODE_SEVEN as u8; 3];
    //     assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0.7 = 1.7\u{1b}[0m\u{1b}[6D".to_string()), d1.dntk_exec(ptr5));
    //     let ptr_enter: [libc::c_char; 3] = [util::ASCII_CODE_NEWLINE as u8; 3];
    //     assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_enter));
    // }

    #[test]
    #[cfg(all(not(target_os = "windows"), not(target_arch = "aarch64")))]
    fn test_dntk_exec() {
        let d1 = &mut Dntker::new();
        let ptr_escape: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as i8; 3];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_escape));
        let ptr1: [libc::c_char; 3] = [util::ASCII_CODE_ONE as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr1));
        let ptr_right: [libc::c_char; 3] = [util::ASCII_CODE_ESCAPE as i8, 0x91 as u8 as i8, util::ASCII_CODE_RIGHT as i8];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_right));
        let ptr2: [libc::c_char; 3] = [util::ASCII_CODE_PLUS as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[35m\r(dntk): 1+ = \u{1b}[0m\u{1b}[3D".to_string()), d1.dntk_exec(ptr2));
        let ptr3: [libc::c_char; 3] = [util::ASCII_CODE_ZERO as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0 = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr3));
        let ptr4: [libc::c_char; 3] = [util::ASCII_CODE_DOT as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr4));
        let ptr_unknown_ascii: [libc::c_char; 3] = [0x4f as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0. = 1\u{1b}[0m\u{1b}[4D".to_string()), d1.dntk_exec(ptr_unknown_ascii));
        let ptr5: [libc::c_char; 3] = [util::ASCII_CODE_SEVEN as i8; 3];
        assert_eq!(DntkResult::Output("\u{1b}[36m\r(dntk): 1+0.7 = 1.7\u{1b}[0m\u{1b}[6D".to_string()), d1.dntk_exec(ptr5));
        let ptr_enter: [libc::c_char; 3] = [util::ASCII_CODE_NEWLINE as i8; 3];
        assert_eq!(DntkResult::Fin, d1.dntk_exec(ptr_enter));
    }

    #[test]
    fn test_colorize() {
        let s = "\r(dntk): 1+2 = 3";
        let ds1 = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Ok,
            cur_pos_from_right: 4,
        };
        let ds2 = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Ng,
            cur_pos_from_right: 4,
        };
        let ds3 = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Warn,
            cur_pos_from_right: 4,
        };
        assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), ds1.colorize().data);
        assert_eq!("\u{1b}[35m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), ds2.colorize().data);
        assert_eq!("\u{1b}[33m\r(dntk): 1+2 = 3\u{1b}[0m".to_string(), ds3.colorize().data);
    }

    #[test]
    fn test_cursorize() {
        let s = "\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m";
        let ds = DntkString {
            data: s.to_string(),
            dtype: DntkStringType::Ok,
            cur_pos_from_right: 4,
        };
        assert_eq!("\u{1b}[36m\r(dntk): 1+2 = 3\u{1b}[0m\u{1b}[4D".to_string(), ds.cursorize().data);
    }
}
