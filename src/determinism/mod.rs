pub mod clock;

#[must_use]
pub fn stable_sort<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values
}
