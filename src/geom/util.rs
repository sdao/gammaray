/// Rearranges the items in the slice such that all items for which the predicate is true come
/// before all elements for which the predicate is false. Returns the index of the first item for
/// which the predicate is false.
pub fn partition<T>(slice: &mut [T], predicate: &Fn(&T) -> bool) -> usize {
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

/// Rearranges the items in the slice such that, in the new configuration, all items before
/// the nth item are less than the nth item, and all items after the nth item are greater than or
/// equal to the nth item. The item in the nth position will be the nth smallest item in the
/// slice.
pub fn nth_element<T>(slice: &mut [T], nth: usize, less_than: &Fn(&T, &T) -> bool) {
    if slice.len() < 2 {
        return;
    }

    let slice_len = slice.len();
    let pivot = slice_len / 2;
    slice.swap(pivot, slice_len - 1);

    // Partition the slice so that all items before i are less than slice[i], and all items
    // after i are greater than or equal to slice[i];
    let final_pivot_index = {
        let (pivot_val, work) = slice.split_last_mut().unwrap();
        partition(work, &|x| less_than(x, pivot_val))
    };
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
