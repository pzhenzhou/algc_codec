/// KMP string matching, complexity is O(m+n)
/// Returns the first matching position
pub fn kmp_search(input: &[char], pattern: &[char]) -> i32 {
    let m = pattern.len();
    if m == 0 {
        return 0;
    }
    if input.len() < pattern.len() {
        return -1;
    }
    let next = next(pattern);
    let mut i = 0;
    let mut j = 0;
    while i < input.len() && j < pattern.len() {
        if input[i] == pattern[j] {
            i += 1;
            j += 1;
        } else if j != 0 {
            j = next[j - 1];
        } else {
            i += 1;
        }
    }
    // println!("i = {}, j = {}", i, j);
    if j == pattern.len() {
        (i - j) as i32
    } else {
        -1
    }
}

/// Calculate Partial Match Table
/// The value in PMT is the length of the longest element in the intersection of
/// the set of prefixes and the set of suffixes of the string
///
/// The process of finding the next array can be seen as the process of string matching, that is,
/// the pattern string is the main string and the prefix of the pattern string is the target string,
/// once the string is matched successfully, then the current next value is the length of
/// the successfully matched string.
fn next(str: &[char]) -> Vec<usize> {
    let mut next_vec = vec![0_usize; str.len()];
    let mut i = 1;
    let mut j = 0;
    while i < str.len() {
        if str[i] == str[j] {
            j += 1;
            next_vec[i] = j;
            i += 1;
        } else if j != 0 {
            j = next_vec[j - 1];
        } else {
            next_vec[i] = 0;
            i += 1;
        }
    }
    next_vec
}
