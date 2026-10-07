//! A set of digits from 1 to 9, such as a cell's candidates.

use std::fmt;

/// Bit 0 is digit 1, bit 8 is digit 9.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Digits(u16);

impl Digits {
    pub const NONE: Digits = Digits(0);
    pub const ALL: Digits = Digits(0x1FF);

    pub fn contains(self, digit: u8) -> bool {
        self.0 & bit(digit) != 0
    }

    pub fn insert(&mut self, digit: u8) {
        self.0 |= bit(digit);
    }

    pub fn remove(&mut self, digit: u8) {
        self.0 &= !bit(digit);
    }

    pub fn toggle(&mut self, digit: u8) {
        self.0 ^= bit(digit);
    }

    pub fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn union(self, other: Digits) -> Digits {
        Digits(self.0 | other.0)
    }

    pub fn without(self, other: Digits) -> Digits {
        Digits(self.0 & !other.0)
    }

    pub fn intersection(self, other: Digits) -> Digits {
        Digits(self.0 & other.0)
    }

    /// The digit, if the set holds exactly one.
    pub fn only(self) -> Option<u8> {
        (self.len() == 1).then(|| self.0.trailing_zeros() as u8 + 1)
    }

    /// Smallest digit first.
    pub fn iter(self) -> impl Iterator<Item = u8> {
        (1..=9).filter(move |&d| self.contains(d))
    }
}

/// Panics on a digit outside 1 to 9, which is a bug in the caller.
fn bit(digit: u8) -> u16 {
    assert!((1..=9).contains(&digit), "digit {digit} is not 1 to 9");
    1 << (digit - 1)
}

impl FromIterator<u8> for Digits {
    fn from_iter<I: IntoIterator<Item = u8>>(digits: I) -> Digits {
        let mut set = Digits::NONE;
        for d in digits {
            set.insert(d);
        }
        set
    }
}

impl fmt::Debug for Digits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for d in self.iter() {
            write!(f, "{d}")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_remove_toggle() {
        let mut set = Digits::NONE;
        set.insert(1);
        set.insert(9);
        set.toggle(5);
        set.toggle(1);
        set.remove(7);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![5, 9]);
        assert_eq!(set.len(), 2);
        assert!(!set.is_empty());
        assert_eq!(format!("{set:?}"), "{59}");
    }

    #[test]
    fn set_operations() {
        let a: Digits = [1, 2, 3].into_iter().collect();
        let b: Digits = [3, 4].into_iter().collect();
        assert_eq!(a.union(b), [1, 2, 3, 4].into_iter().collect());
        assert_eq!(a.without(b), [1, 2].into_iter().collect());
        assert_eq!(a.intersection(b), [3].into_iter().collect());
        assert_eq!(b.without(a).only(), Some(4));
        assert_eq!(a.only(), None);
        assert_eq!(Digits::NONE.only(), None);
        assert_eq!(Digits::ALL.len(), 9);
    }

    #[test]
    #[should_panic(expected = "digit 10 is not 1 to 9")]
    fn rejects_digits_outside_one_to_nine() {
        Digits::NONE.contains(10);
    }
}
