pub const HALT: u8 = 0b111111;
pub const PUSHENV: u8 = 1;
pub const POPENV: u8 = 1;
pub const GETVAR: u8 = 2;
pub const SETVAR: u8 = 3;
pub const NEWENV: u8 = 4;
pub const GETELEM: u8 = 5;
pub const SETELEM: u8 = 6;
pub const PUSHLIT: u8 = 7;
pub const CALL: u8 = 8;
pub const RET: u8 = 9;
pub const POPVAL: u8 = 10;

pub const ADD: u8 = 16;
pub const SUB: u8 = 17;
pub const MUL: u8 = 18;
pub const DIV: u8 = 19;

pub const TEST: u8 = 20;

pub const JMP: u8 = 32;
pub const JT: u8 = 33;
pub const JF: u8 = 34;
