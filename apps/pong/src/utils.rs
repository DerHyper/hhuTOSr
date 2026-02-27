/// Round to the neares int.<br>
/// Needed because `f32::round()` cannot be used, as it requires std.<br>
/// Example:<br>
/// 0.3 -> 0<br>
/// 0.5 -> 1<br>
/// 0.7 -> 1<br>
pub fn round(n: f32) -> usize {
    let mut result = n as usize;
    if n%1.0 >= 0.5 {
        result = result+1;
    }
    return result;
}

/// Square of a number
pub fn square(n: f32) -> f32 {
    n*n
}

/// Square root of a number.<br>
/// Needed because `f32::sqrt()` cannot be used, as it requires std.
pub fn sqrt(n: f32) -> f32 {
    if n == 0.0 {
        return 0.0;
    }
    let mut x = n;
    let mut y = 1.0;
    while (x - y).abs() > 0.001 {
        x = (x + y) / 2.0;
        y = n / x;
    }
    return x;
}

