use std;

pub const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

pub fn clamp<T>(x: T, a: T, b: T) -> T where T: PartialOrd {
    if x < a {
        a
    }
    else if x > b {
        b
    }
    else {
        x
    }
}

pub fn clamp_unit(x: f32) -> f32 {
    clamp(x, 0.0, 1.0)
}

/**
 * Linearly interpolates between x and y. Where a = 0, x is returned, and
 * where a = 1, y is returned. If a < 0 or a > 1, this function will
 * extrapolate.
 */
pub fn lerp(x: f32, y: f32, a: f32) -> f32 {
    x + a * (y - x)
}

/**
 * Linearly interpolates between x and y. Where a <= 0, x is returned, and
 * where a >= 1, y is returned. No extrapolation will occur.
 */
pub fn clamped_lerp(x: f32, y: f32, a: f32) -> f32 {
    lerp(x, y, clamp(a, 0.0, 1.0))
}

/**
 * Determines whether a number is zero, within a small epsilon.
 */
pub fn is_nearly_zero(x: f32) -> bool {
    f32::abs(x) < std::f32::EPSILON
}

/**
 * Determines whether a number is positive, within a small epsilon.
 */
pub fn is_positive(x: f32) -> bool {
    x > std::f32::EPSILON
}

/**
 * Determines whether two numbers are close, within a user-provided epsilon.
 */
pub fn is_close(x: f32, y: f32, eps: f32) -> bool {
    f32::abs(x - y) < eps
}

/**
 * Evaluates a triangle filter with width = 0.5 (support = 1.0) for a
 * specified offset from the pixel center. The values are not normalized,
 * i.e., the integral of the filter over the 1x1 square around the point.
 * Thus, you should only use the filter weights relative to other weights.
 *
 * In fact, Mathematica says that:
 * @code
 * In := Integrate[(0.5-Abs[x])*(0.5-Abs[y]), {x, -0.5, 0.5}, {y, -0.5, 0.5}]
 * Out = 0.0625
 * @endcode
 *
 * @param x     the x-offset from the pixel center, -width <= x <= width
 * @param y     the y-offset from the pixel center, -width <= x <= width
 * @param width the maximum x- or y- offset sampled from the pixel center
 *              (A recommended default width is 2.0)
 * @returns the value of the filter
 */
pub fn triangle_filter(x: f32, y: f32, width: f32) -> f32 {
    f32::max(0.0, width - f32::abs(x)) * f32::max(0.0, width - f32::abs(y))
}

/**
 * Computes the 1-dimensional Mitchell filter with B = 1/3 and C = 1/3 for a
 * scaled offset from the pixel center. The values are not normalized.
 *
 * Pharr and Humphreys suggest on p. 398 of PBR that values of B and C should
 * be chosen such that B + 2C = 1.
 * GPU Gems <http://http.developer.nvidia.com/GPUGems/gpugems_ch24.html>
 * suggests the above values of B = 1/3 and C = 1/3.
 *
 * @param x the scaled x-offset from the pixel center, -1 <= x <= 1
 */
pub fn mitchell_filter1(x: f32) -> f32 {
    const B: f32 = 1.0 / 3.0;
    const C: f32 = 1.0 / 3.0;

    let twox = f32::abs(2.0 * x); // Convert to the range [0, 2].

    if twox > 1.0 {
        ((-B - 6.0 * C) * (twox * twox * twox)
        + (6.0 * B + 30.0 * C) * (twox * twox)
        + (-12.0 * B - 48.0 * C) * twox
        + (8.0 * B + 24.0 * C)) * (1.0 / 6.0)
    } else {
        ((12.0 - 9.0 * B - 6.0 * C) * (twox * twox * twox)
        + (-18.0 + 12.0 * B + 6.0 * C) * (twox * twox)
        + (6.0 - 2.0 * B)) * (1.0 / 6.0)
    }
}

/**
 * Evaluates a 2-dimensional Mitchell filter at a specified offset from the
 * pixel center by separating and computing the 1-dimensional Mitchell
 * filter for the x- and y- offsets.
 *
 * @param x     the x-offset from the pixel center, -width <= x <= width
 * @param y     the y-offset from the pixel center, -width <= x <= width
 * @param width the maximum x- or y- offset sampled from the pixel center
 *              (A recommended default width is 2.0)
 * @returns the value of the filter
 */
pub fn mitchell_filter2(x: f32, y: f32, width: f32) -> f32 {
    mitchell_filter1(x / width) * mitchell_filter1(y / width)
}

/**
 * Calculates the power heuristic for multiple importance sampling of
 * two separate functions.
 *
 * See Pharr & Humphreys p. 693 for more information.
 *
 * @param nf   number of samples taken with a Pf distribution
 * @param fPdf probability according to the Pf distribution
 * @param ng   number of samples taken with a Pg distribution
 * @param gPdf probability according to the Pg distribution
 * @returns    the weight according to the power heuristic
 */
pub fn power_heuristic(nf: u32, f_pdf: f32, ng: u32, g_pdf: f32) -> f32{
    let f = (nf as f32) * f_pdf;
    let g = (ng as f32) * g_pdf;

    (f * f) / (f * f + g * g)
}

pub fn row_col(index: usize, width: usize) -> (usize, usize) {
    (index / width, index % width)
}

pub fn index(row: usize, col: usize, width: usize) -> usize {
    row * width + col
}

pub fn gamma(n: f32) -> f32 {
    (n * std::f32::EPSILON) / (1.0 - (n * std::f32::EPSILON))
}
