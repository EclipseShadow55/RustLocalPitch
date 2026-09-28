use std::cmp::{Ordering};
use std::collections::Bound;
use std::ops::RangeBounds;
use num_traits::Num;

trait CheckBounds<T> where T: PartialOrd {
    fn check_bounds(&self, val: T) -> bool;
}

pub trait Parameter<T>: PartialOrd<T> + PartialEq<T> where T: PartialOrd + PartialEq {
    fn shift_up(&self) -> T;
    fn shift_down(&self) -> T;
    fn set(&mut self, val: T);
    fn shift_up_bounded(&self, maximum: T) -> Result<T, ()> {
        if self.shift_up() > maximum {
            if self.get() != maximum {
                Ok(maximum)
            } else {
                Err(())
            }
        } else {
            Ok(self.shift_up())
        }
    }
    fn shift_down_bounded(&self, minimum: T) -> Result<T, ()> {
        if self.shift_down() < minimum {
            if self.get() != minimum {
                Ok(minimum)
            } else {
                Err(())
            }
        } else {
            Ok(self.shift_down())
        }
    }
    fn get(&self) -> T;
    fn default(&self) -> T;
}

pub trait BoundedParameter<T> {
    fn shift_up(&self) -> Result<T, ()>;
    fn shift_down(&self) -> Result<T, ()>;
    fn set(&mut self, val: T);
    fn get(&self) -> T;
    fn default(&self) -> T;
    fn lower_bound(&self) -> Bound<T>;
    fn upper_bound(&self) -> Bound<T>;
}

pub trait ParamType: Copy + Num + PartialOrd {}

impl<T> ParamType for T where T: Copy + Num + PartialOrd {}


pub struct LinearParam<T> where T: ParamType {
    default: T,
    current: T,
    shift: T,
}

impl<T> LinearParam<T> where T: ParamType {
    pub fn new(val: T, shift: T) -> Self {
        Self {
            default: val,
            current: val,
            shift
        }
    }
}

impl<T> PartialEq<T> for LinearParam<T> where T: ParamType {
    fn eq(&self, other: &T) -> bool {
        self.current == *other
    }
}

impl<T> PartialOrd<T> for LinearParam<T> where T: ParamType {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.current == *other {
            Some(Ordering::Equal)
        } else if self.current < *other {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
    fn lt(&self, other: &T) -> bool {
        self.current < *other
    }
    fn le(&self, other: &T) -> bool {
        self.current <= *other
    }
    fn gt(&self, other: &T) -> bool {
        self.current > *other
    }
    fn ge(&self, other: &T) -> bool {
        self.current >= *other
    }
}

impl<T> Parameter<T> for LinearParam<T> where T: ParamType {
    fn shift_up(&self) -> T {
        self.current + self.shift
    }
    fn shift_down(&self) -> T {
        self.current - self.shift
    }
    fn set(&mut self, val: T) {
        self.current = val;
    }
    fn get(&self) -> T {
        self.current
    }
    fn default(&self) -> T {
        self.default
    }
}


pub struct ExponentialParam<T> where T: ParamType {
    default: T,
    current: T,
    shift: T,
}

impl<T> ExponentialParam<T> where T: ParamType {
    pub fn new(val: T, shift: T) -> Self {
        Self {
            default: val,
            current: val,
            shift
        }
    }
}

impl<T> PartialEq<T> for ExponentialParam<T> where T: ParamType {
    fn eq(&self, other: &T) -> bool {
        self.current == *other
    }
}

impl<T> PartialOrd<T> for ExponentialParam<T> where T: ParamType {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.current == *other {
            Some(Ordering::Equal)
        } else if self.current < *other {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
    fn lt(&self, other: &T) -> bool {
        self.current < *other
    }
    fn le(&self, other: &T) -> bool {
        self.current <= *other
    }
    fn gt(&self, other: &T) -> bool {
        self.current > *other
    }
    fn ge(&self, other: &T) -> bool {
        self.current >= *other
    }
}

impl<T> Parameter<T> for ExponentialParam<T> where T: ParamType {
    fn shift_up(&self) -> T {
        self.current * self.shift
    }
    fn shift_down(&self) -> T {
        self.current / self.shift
    }
    fn set(&mut self, val: T) {
        self.current = val;
    }
    fn get(&self) -> T {
        self.current
    }
    fn default(&self) -> T {
        self.default
    }
}


pub struct CyclicParam<T> where T: ParamType {
    default: T,
    current: T,
    shift: T,
    top: T,
    bottom: T,
    span: T,
}

impl<T> CyclicParam<T> where T: ParamType {
    pub fn new(val: T, shift: T, top: T, bottom: T) -> Result<Self, ()> {
        if top <= bottom {
            Err(())
        } else {
            Ok(Self {
                default: val,
                current: val,
                shift,
                top,
                bottom,
                span: top - bottom
            })
        }
    }
}

impl<T> PartialEq<T> for CyclicParam<T> where T: ParamType {
    fn eq(&self, other: &T) -> bool {
        self.current == *other
    }
}

impl<T> PartialOrd<T> for CyclicParam<T> where T: ParamType {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.current == *other {
            Some(Ordering::Equal)
        } else if self.current < *other {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
    fn lt(&self, other: &T) -> bool {
        self.current < *other
    }
    fn le(&self, other: &T) -> bool {
        self.current <= *other
    }
    fn gt(&self, other: &T) -> bool {
        self.current > *other
    }
    fn ge(&self, other: &T) -> bool {
        self.current >= *other
    }
}

impl<T> Parameter<T> for CyclicParam<T> where T: ParamType {
    fn shift_up(&self) -> T {
        if self.top - self.current <= self.shift {
            self.bottom + (self.shift - (self.top - self.current))
        } else {
            self.current + self.shift
        }
    }
    fn shift_down(&self) -> T {
        if self.current - self.bottom < self.shift {
            self.top - (self.shift - (self.current - self.bottom))
        } else {
            self.current - self.shift
        }
    }
    fn set(&mut self, val: T) {
        self.current = val;
    }
    fn get(&self) -> T {
        self.current
    }
    fn default(&self) -> T {
        self.default
    }
}

pub struct BoolParam {
    default: bool,
    current: bool,
}

impl BoolParam {
    pub fn new(val: bool) -> Self {
        Self {
            default: val,
            current: val,
        }
    }

    pub fn toggle(&self) -> bool {
        !self.current
    }
}

impl PartialEq<bool> for BoolParam where {
    fn eq(&self, other: &bool) -> bool {
        self.current == *other
    }
}

impl PartialOrd<bool> for BoolParam where {
    fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
        if self.current == *other {
            Some(Ordering::Equal)
        } else if self.current < *other {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
    fn lt(&self, other: &bool) -> bool {
        self.current < *other
    }
    fn le(&self, other: &bool) -> bool {
        self.current <= *other
    }
    fn gt(&self, other: &bool) -> bool {
        self.current > *other
    }
    fn ge(&self, other: &bool) -> bool {
        self.current >= *other
    }
}

impl Parameter<bool> for BoolParam {
    fn shift_up(&self) -> bool {
        self.toggle()
    }
    fn shift_down(&self) -> bool {
        self.toggle()
    }
    fn set(&mut self, val: bool) {
        self.current = val;
    }
    fn get(&self) -> bool {
        self.current
    }
    fn default(&self) -> bool {
        self.default
    }
}

pub struct BoundedParam<C, T> where C: Parameter<T>, T: ParamType {
    inner: C,
    lower_bound: Bound<T>,
    upper_bound: Bound<T>,
}

impl<C, T> BoundedParam<C, T> where C: Parameter<T>, T: ParamType {
    pub fn new(inner: C, lower_bound: Bound<T>, upper_bound: Bound<T>) -> Self {
        Self {
            inner,
            lower_bound,
            upper_bound
        }
    }
}

impl<C, T> PartialEq<T> for BoundedParam<C, T> where C: Parameter<T>, T: ParamType {
    fn eq(&self, other: &T) -> bool {
        self.get() == *other
    }
}

impl<C, T> PartialOrd<T> for BoundedParam<C, T> where C: Parameter<T>, T: ParamType {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        if self.get() == *other {
            Some(Ordering::Equal)
        } else if self.get() < *other {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
    fn lt(&self, other: &T) -> bool {
        self.get() < *other
    }
    fn le(&self, other: &T) -> bool {
        self.get() <= *other
    }
    fn gt(&self, other: &T) -> bool {
        self.get() > *other
    }
    fn ge(&self, other: &T) -> bool {
        self.get() >= *other
    }
}

impl<C, T> BoundedParameter<T> for BoundedParam<C, T> where C: Parameter<T>, T: ParamType {
    fn shift_up(&self) -> Result<T, ()> {
        if (Bound::Unbounded::<T>, self.upper_bound).contains(&self.inner.shift_up()) {
            Ok(self.inner.shift_up())
        } else {
            if let Bound::Included(num) = self.upper_bound && self.inner.get() != num {
                Ok(num)
            } else {
                Err(())
            }
        }
    }
    fn shift_down(&self) -> Result<T, ()> {
        if (self.lower_bound, Bound::Unbounded::<T>).contains(&self.inner.shift_down()) {
            Ok(self.inner.shift_down())
        } else {
            if let Bound::Included(num) = self.lower_bound && self.inner.get() != num {
                Ok(num)
            } else {
                Err(())
            }
        }
    }
    fn set(&mut self, val: T) {
        self.inner.set(val);
    }
    fn get(&self) -> T {
        self.inner.get()
    }
    fn default(&self) -> T {
        self.inner.default()
    }
    fn lower_bound(&self) -> Bound<T> {
        self.lower_bound
    }
    fn upper_bound(&self) -> Bound<T> {
        self.upper_bound
    }
}