/**
 * Take a string num representation f.e 1234 or 0x1234
 * Strip 0x or 0X prefix if present
 * Parse the value and returnf if successful; otherwise None
 */
pub fn parse_value(raw: &str) -> Option<i32> {
	let raw = raw.trim();

	match raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
		Some(hex) => i32::from_str_radix(hex, 16).ok(),
		None => raw.parse().ok(),
	}
}
