use rand::seq::SliceRandom;



pub struct ReservoirIter<T, I> {
    reservoir: Vec<T>,
    max_size: usize,

    inner: I,
}

impl<T, I> ReservoirIter<T, I> {
    pub fn new(inner: I, max_size: usize) -> Self {
        Self {
            reservoir: Default::default(),
            max_size,
            inner,
        }
    }
}

impl<T, I> Iterator for ReservoirIter<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // Fill the reservoir
        while self.reservoir.len() < self.max_size {
            match self.inner.next() {
                Some(val) => self.reservoir.push(val),
                None => break,
            }
        }

        // Pick a random item
        self.reservoir.shuffle(&mut rand::rng());

        self.reservoir.pop()
    }
}
