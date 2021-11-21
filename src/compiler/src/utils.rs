use std::collections::VecDeque;

#[derive(Debug)]
pub struct LimitedVecDeque<T> {
    capacity: usize,
    deque: VecDeque<T>,
}

impl<T> LimitedVecDeque<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            deque: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: T) {
        if self.deque.len() == self.capacity {
            self.deque.pop_front();
        }
        self.deque.push_back(value)
    }
}
