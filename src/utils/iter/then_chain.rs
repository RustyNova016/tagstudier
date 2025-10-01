use rand::seq::SliceRandom;

pub struct ThenChainIter<I, I2, F> {
    a: Option<I>,
    b: Option<I2>,
    func: Option<F>,
}

impl<I, I2, F> ThenChainIter<I, I2, F> {
    pub fn new(inner: I, func: F) -> Self {
        Self {
            a: Some(inner),
            b: None,
            func: Some(func),
        }
    }
}

impl<I, I2, F, T> Iterator for ThenChainIter<I, I2, F>
where
    I: Iterator<Item = T>,
    I2: Iterator<Item = T>,
    F: FnOnce() -> I2,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let a_res = self.a.as_mut().and_then(Iterator::next);

        if a_res.is_none() {
            self.a = None;
        } else {
            return a_res;
        }

        if let Some(func) = self.func.take() {
            self.b = Some((func)());
        }

        let b_res = self.b.as_mut().and_then(Iterator::next);

        if b_res.is_none() {
            self.b = None;
        }

        b_res
    }
}
