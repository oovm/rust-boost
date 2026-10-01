use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub struct Show<T>(pub T);

impl<T: Display> Display for Show<Option<T>> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self.0 {
            Some(x) => write!(f, "{}", x),
            None => Ok(()),
        }
    }
}

impl<'a, T, F: Fn(&mut Formatter, &'a T) -> std::fmt::Result> Display for Show<(&'a [T], F)> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for x in self.0.0 {
            (self.0.1)(f, x)?;
        }
        Ok(())
    }
}

impl<T: Display> Display for Show<(T, usize)> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for _ in 0..self.0.1 {
            write!(f, "{}", self.0.0)?;
        }
        Ok(())
    }
}
