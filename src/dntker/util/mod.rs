pub const DNTK_PROMPT            : &str = "\r(dntk): ";

// https://qiita.com/hidai@github/items/1704bf2926ab8b157a4f
#[allow(dead_code)]
pub const COLOR_BLACK_HEADER     : &str = "\x1b[30m";
#[allow(dead_code)]
pub const COLOR_RED_HEADER       : &str = "\x1b[31m";
#[allow(dead_code)]
pub const COLOR_GREEN_HEADER     : &str = "\x1b[32m";
#[allow(dead_code)]
pub const COLOR_YELLOW_HEADER    : &str = "\x1b[33m";
#[allow(dead_code)]
pub const COLOR_BLUE_HEADER      : &str = "\x1b[34m";
pub const COLOR_MAGENDA_HEADER   : &str = "\x1b[35m";
pub const COLOR_CYAN_HEADER      : &str = "\x1b[36m";
#[allow(dead_code)]
pub const COLOR_WHITE_HEADER     : &str = "\x1b[37m";
pub const COLOR_PLAIN_HEADER     : &str = "\x1b[0m";

// http://www9.plala.or.jp/sgwr-t/c_sub/ascii.html
pub const ASCII_CODE_ZERO        : u8 = 0x30; // 0
pub const ASCII_CODE_ONE         : u8 = 0x31; // 1
pub const ASCII_CODE_TWO         : u8 = 0x32; // 2
pub const ASCII_CODE_THREE       : u8 = 0x33; // 3
pub const ASCII_CODE_FOUR        : u8 = 0x34; // 4
pub const ASCII_CODE_FIVE        : u8 = 0x35; // 5
pub const ASCII_CODE_SIX         : u8 = 0x36; // 6
pub const ASCII_CODE_SEVEN       : u8 = 0x37; // 7
pub const ASCII_CODE_EIGHT       : u8 = 0x38; // 8
pub const ASCII_CODE_NINE        : u8 = 0x39; // 9
pub const ASCII_CODE_S           : u8 = 0x73; // s
pub const ASCII_CODE_C           : u8 = 0x63; // c
pub const ASCII_CODE_A           : u8 = 0x61; // a
pub const ASCII_CODE_L           : u8 = 0x6c; // l
pub const ASCII_CODE_E           : u8 = 0x65; // e
pub const ASCII_CODE_J           : u8 = 0x6a; // j
pub const ASCII_CODE_R           : u8 = 0x72; // r
pub const ASCII_CODE_Q           : u8 = 0x71; // q
pub const ASCII_CODE_T           : u8 = 0x74; // t
pub const ASCII_CODE_ROUNDLEFT   : u8 = 0x28; // (
pub const ASCII_CODE_ROUNDRIGHT  : u8 = 0x29; // )
pub const ASCII_CODE_SQUARELEFT  : u8 = 0x5b; // [
pub const ASCII_CODE_SQUARERIGHT : u8 = 0x5d; // ]
pub const ASCII_CODE_LARGER      : u8 = 0x3c; // <
pub const ASCII_CODE_SMALLER     : u8 = 0x3e; // >
pub const ASCII_CODE_RIGHT       : u8 = 0x43; // →
pub const ASCII_CODE_LEFT        : u8 = 0x44; // ←
pub const ASCII_CODE_PLUS        : u8 = 0x2b; // +
pub const ASCII_CODE_MINUS       : u8 = 0x2d; // -
pub const ASCII_CODE_ASTERISK    : u8 = 0x2a; // *
pub const ASCII_CODE_SLUSH       : u8 = 0x2f; // /
pub const ASCII_CODE_HAT         : u8 = 0x5e; // ^
pub const ASCII_CODE_PERCENT     : u8 = 0x25; // %
pub const ASCII_CODE_DOT         : u8 = 0x2e; // .
pub const ASCII_CODE_BIKKURI     : u8 = 0x21; // !
pub const ASCII_CODE_EQUAL       : u8 = 0x3d; // =
pub const ASCII_CODE_PIPE        : u8 = 0x7c; // |
pub const ASCII_CODE_AND         : u8 = 0x26; // &
pub const ASCII_CODE_SEMICOLON   : u8 = 0x3b; // ;
pub const ASCII_CODE_AT          : u8 = 0x40; // @
pub const ASCII_CODE_NEWLINE     : u8 = 0x0a; // \n
pub const ASCII_CODE_ESCAPE      : u8 = 0x1b; // escape key
pub const ASCII_CODE_BACKSPACE   : u8 = 0x08; // backspace key
pub const ASCII_CODE_DELETE      : u8 = 0x7f; // delete key
pub const ASCII_CODE_SPACE       : u8 = 0x20; // white space key

// http://tldp.org/HOWTO/Bash-Prompt-HOWTO/x361.html
pub const CURSOR_MOVE_ES_HEAD    : &str = "\x1b[";
pub const CURSOR_MOVE_ES_BACK    : &str = "D";