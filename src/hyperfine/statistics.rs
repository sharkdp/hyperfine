/// Calculate statistical average
pub fn mean<I>(values: I) -> f64
where
    I: IntoIterator<Item = f64>,
{
    let mut sum: f64 = 0.0;
    let mut len: u64 = 0;
    for v in values {
        sum += v;
        len += 1;
    }
    sum / (len as f64)
}
