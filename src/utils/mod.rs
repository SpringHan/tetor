// Utils

use std::mem::swap;

pub fn cursor_compare_swap<T>(small: &mut (T, T), big: &mut (T, T))
where T: PartialEq + PartialOrd + Copy
{
    if small.1 > big.1 {
        swap(small, big);
    }

    if small.1 == big.1 && small.0 > big.0 {
        swap(small, big);
    }
}
