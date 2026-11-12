// 0x1234
// 0x12  - instruction code (high byte)
// 34    - operand (low byte)
pub fn encode(code: i32, operand: i32) -> i32 {
    ((code & 0xff) << 8) | (operand & 0xff)
}
