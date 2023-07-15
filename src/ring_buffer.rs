use std::collections::VecDeque;

#[derive(Debug)]
struct RingBuffer<T> {
    inner: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        if self.inner.len() == self.inner.capacity() {
            self.inner.pop_front();
        }
        self.inner.push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }
}

impl<T> Iterator for RingBuffer<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut buf = RingBuffer::new(3);

        buf.push(3);
        buf.push(4);
        buf.push(5);
        buf.push(1);

        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_iter() {
        let mut buf = RingBuffer::new(3);

        buf.push(3.1);
        buf.push(4.2);
        buf.push(5.3);
        buf.push(1.4);

        assert_eq!(buf.next(), Some(4.2));
        assert_eq!(buf.next(), Some(5.3));
        assert_eq!(buf.next(), Some(1.4));
        assert_eq!(buf.next(), None);
    }
}
