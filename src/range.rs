use std::ops::Range;

pub trait Membership<T> {
    fn inclusive(&self, value: T) -> bool;
    fn exclusive(&self, value: T) -> bool;
}

impl Membership<f64> for Range<f64> {
    // check if value is in range, inclusive
    fn inclusive(&self, value: f64) -> bool {
        self.start <= value && value <= self.end
    }
    // check if value is in range, exclusive
    fn exclusive(&self, value: f64) -> bool {
        self.start < value && value < self.end
    }
}

pub trait Expandable<T> {
    fn expand(&self, value: T) -> Self;
    fn union(&self, other: &Self) -> Self;
}

impl Expandable<f64> for Range<f64> {
    fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.;
        (self.start - padding)..(self.end + padding)
    }
    fn union(&self, other: &Self) -> Self {
        self.start.min(other.start)..self.end.max(other.end)
    }
}

pub trait RangeExtensions<T> {
    fn middle(&self) -> T;
}

impl RangeExtensions<f64> for Range<f64> {
    fn middle(&self) -> f64 {
        (self.start + self.end) / 2.
    }
}
