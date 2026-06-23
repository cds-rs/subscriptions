/// Seconds in `mins` minutes.
pub fn minutes(mins: u64) -> u64 {
    mins * 60
}

/// Seconds in `d` days.
pub fn days(d: u64) -> u64 {
    d * 24 * 60 * 60
}

/// Rent-exempt minimum for `space` bytes, computed neutrally (no engine). The
/// engine-agnostic replacement for `LiteSVM::minimum_balance_for_rent_exemption`.
pub fn rent_exempt_lamports(space: usize) -> u64 {
    solana_rent::Rent::default().minimum_balance(space)
}
