use core::fmt;
use core::ops::{Add, AddAssign, Sub};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub fn offset_from(&self, other: Self) -> usize {
        self.0
            .checked_sub(other.0)
            .expect("Overflow when calculating address offset")
    }
}

impl From<usize> for PhysicalAddress {
    fn from(address: usize) -> Self {
        Self(address)
    }
}

impl From<PhysicalAddress> for usize {
    fn from(address: PhysicalAddress) -> Self {
        address.0
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

// Implements `PhysicalAddress - usize`
impl Sub<usize> for PhysicalAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

// Implements `PhysicalAddress - PhysicalAddress`
impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = usize;

    fn sub(self, rhs: PhysicalAddress) -> Self::Output {
        self.0 - rhs.0
    }
}

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0) // hex
    }
}
