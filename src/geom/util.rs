pub fn partition<T>(slice: &mut [T], predicate: &Fn(&T) -> bool) -> usize {
    if slice.len() == 0 {
        return 0
    }

    let slice_len = slice.len();
    let mut cursor = 0;
    for i in 0..slice_len {
        if predicate(&slice[i]) {
            slice.swap(i, cursor);
            cursor += 1;
        }
    }

    cursor
}

pub fn nth_element<T>(slice: &mut [T], nth: usize, less_than: &Fn(&T, &T) -> bool) {
    if slice.len() < 2 {
        return;
    }

    let slice_len = slice.len();
    let pivot = slice_len / 2;
    slice.swap(pivot, slice_len - 1);

    // Partition the slice so that all items before i are less than slice[i], and all items
    // after i are greater than or equal to slice[i];
    let mut final_pivot_index = 0;
    for i in 0..(slice_len - 1) {
        if less_than(&slice[i], &slice[slice_len - 1]) {
            slice.swap(i, final_pivot_index);
            final_pivot_index += 1;
        }
    }
    slice.swap(slice_len - 1, final_pivot_index);

    // Choose which side of the slice to recurse into. (If the pivot position is nth, then done!)
    if nth < final_pivot_index {
        nth_element(&mut slice[0..(final_pivot_index - 1)], nth, less_than);
    }
    else if nth > final_pivot_index {
        nth_element(&mut slice[(final_pivot_index + 1)..slice_len], nth - final_pivot_index - 1,
                less_than);
    }
}
