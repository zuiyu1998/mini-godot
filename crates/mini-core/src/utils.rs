pub use rustc_hash::*;

pub fn cmp_strings_case_insensitive(a: impl AsRef<str>, b: impl AsRef<str>) -> bool {
    let a_ref = a.as_ref();
    let b_ref = b.as_ref();

    if a_ref.len() != b_ref.len() {
        return false;
    }

    a_ref
        .chars()
        .zip(b_ref.chars())
        .all(|(ca, cb)| ca.to_lowercase().eq(cb.to_lowercase()))
}
