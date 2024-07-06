use std::iter::Peekable;

pub struct Intersperse<I: Iterator> {
    iter: Peekable<I>,
    val: I::Item,
    do_separator: bool,
}

impl<I> Intersperse<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn new(iter: I, val: I::Item) -> Self {
        Self {
            iter: iter.peekable(),
            val,
            do_separator: false,
        }
    }
}

impl<I> Iterator for Intersperse<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.do_separator {
            self.do_separator = false;
            return Some(self.val.clone());
        }

        if self.iter.peek().is_some() {
            self.do_separator = true;
        }

        self.iter.next()
    }
}
