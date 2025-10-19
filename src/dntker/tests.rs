use super::{util, Dntker, FilterResult};

fn statement_bytes(d: &Dntker) -> Vec<u8> {
    d.statement_from_utf8().into_bytes()
}

#[test]
fn filter_char_classifies_known_inputs() {
    let d: Dntker = Default::default();
    assert_eq!(
        d.filter_char(util::ASCII_CODE_ONE),
        FilterResult::Calculatable(util::ASCII_CODE_ONE)
    );
    assert_eq!(
        d.filter_char(util::ASCII_CODE_PLUS),
        FilterResult::Calculatable(util::ASCII_CODE_PLUS)
    );
    assert_eq!(
        d.filter_char(util::ASCII_CODE_SQUARELEFT),
        FilterResult::CurLeft
    );
    assert_eq!(
        d.filter_char(util::ASCII_CODE_SQUARERIGHT),
        FilterResult::CurRight
    );
    assert_eq!(d.filter_char(util::ASCII_CODE_AT), FilterResult::Refresh);
    assert_eq!(d.filter_char(util::ASCII_CODE_ESCAPE), FilterResult::Esc);
    assert_eq!(d.filter_char(util::ASCII_CODE_DELETE), FilterResult::Delete);
    assert_eq!(d.filter_char(0x00), FilterResult::Unknown(0x00));
}

#[test]
fn delete_column_removes_previous_byte() {
    let d1: &mut Dntker = &mut Default::default();
    d1.delete_column();
    assert!(statement_bytes(d1).is_empty());
    assert_eq!(d1.cursor(), 0);

    d1.insert_column(util::ASCII_CODE_ONE);
    d1.insert_column(util::ASCII_CODE_PLUS);
    d1.insert_column(util::ASCII_CODE_TWO);
    d1.cursor_move_left();
    d1.delete_column();

    assert_eq!(
        statement_bytes(d1),
        vec![util::ASCII_CODE_ONE, util::ASCII_CODE_TWO]
    );
    assert_eq!(d1.cursor(), 1);
}

#[test]
fn cursor_movement_bounds_checks() {
    let d: &mut Dntker = &mut Default::default();
    d.cursor_move_left();
    assert_eq!(d.cursor(), 0);

    d.insert_column(util::ASCII_CODE_ONE);
    d.insert_column(util::ASCII_CODE_PLUS);
    d.insert_column(util::ASCII_CODE_TWO);
    assert_eq!(d.cursor(), 3);

    d.cursor_move_left();
    assert_eq!(d.cursor(), 2);

    d.cursor_move_right();
    assert_eq!(d.cursor(), 3);

    d.cursor_move_right();
    assert_eq!(d.cursor(), 3);
}

#[test]
fn insert_column_inserts_at_cursor() {
    let d: &mut Dntker = &mut Default::default();
    d.insert_column(util::ASCII_CODE_ONE);
    d.insert_column(util::ASCII_CODE_TWO);
    assert_eq!(
        statement_bytes(d),
        vec![util::ASCII_CODE_ONE, util::ASCII_CODE_TWO]
    );

    d.cursor_move_left();
    d.insert_column(util::ASCII_CODE_PLUS);
    assert_eq!(
        statement_bytes(d),
        vec![
            util::ASCII_CODE_ONE,
            util::ASCII_CODE_PLUS,
            util::ASCII_CODE_TWO,
        ]
    );
    assert_eq!(d.cursor(), 2);
}

#[test]
fn refresh_resets_prompt_state() {
    let d: &mut Dntker = &mut Default::default();
    d.insert_column(util::ASCII_CODE_ONE);
    d.insert_column(util::ASCII_CODE_PLUS);
    std::env::set_var("DNTK_ENV", "TEST");
    d.refresh();

    assert!(statement_bytes(d).is_empty());
    assert_eq!(d.cursor(), 0);
    std::env::remove_var("DNTK_ENV");
}
