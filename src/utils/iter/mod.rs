use crate::utils::iter::reservoir_iter::ReservoirIter;
use crate::utils::iter::then_chain::ThenChainIter;

pub mod reservoir_iter;
pub mod then_chain;

pub trait IteratorAdditions<T>: Iterator<Item = T> + Sized {
    /// Randomize the iterator in place. This use a modified reservoir sampling algorythm to let it resolve items as it goes
    ///
    /// # The algorythm
    ///
    /// The iterator will buffer up to `max_size` items, shuffle the inner reservoir, and pop the last element.
    ///
    /// If there's not enough elements to fill the reservoir, it will start shuffling early
    fn reservoir_rand(self, max_size: usize) -> ReservoirIter<T, Self> {
        ReservoirIter::new(self, max_size)
    }

    fn then_chain<F, I2>(self, func: F) -> ThenChainIter<Self, I2, F>
    where
        I2: Iterator<Item = T>,
        F: FnOnce() -> I2,
    {
        ThenChainIter::new(self, func)
    }
}

impl<T, I> IteratorAdditions<T> for I where I: Iterator<Item = T> + Sized {}
