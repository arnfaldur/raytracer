use std::ops::Range;

pub trait Membership<T> {
    fn inclusive(&self, value: T) -> bool;
    fn exclusive(&self, value: T) -> bool;
}

impl Membership<f64> for Range<f64> {
    fn inclusive(&self, value: f64) -> bool {
        self.start <= value && value <= self.end
    }
    fn exclusive(&self, value: f64) -> bool {
        self.start < value && value < self.end
    }
}
