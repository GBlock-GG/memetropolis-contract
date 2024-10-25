use std::ops::{Div, Mul};
use std::str::FromStr;

pub fn convert_to_float(value: u64, decimals: u8) -> f64 {
    (value as f64).div(f64::powf(10.0, decimals as f64))
}

pub fn convert_from_float(value: f64, decimals: u8) -> u64 {
    value.mul(f64::powf(10.0, decimals as f64)) as u64
}

pub fn u8_array_to_string(arr: &[u8]) -> String {
    // Find the position of the last non-zero byte
    if let Some(pos) = arr.iter().rposition(|&x| x != 0) {
        // Convert the trimmed slice into a String, assuming valid UTF-8
        String::from_utf8(arr[..=pos].to_vec()).expect("Invalid UTF-8 sequence")
    } else {
        // If all elements are zero, return an empty string
        String::from_str("MINT").unwrap()
    }
}