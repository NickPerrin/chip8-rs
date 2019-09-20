use std::io::{Error, ErrorKind};
use std::vec::Vec;

/// A simple fixed-size stack implementation.
pub struct Stack<T> {
    pub size: usize,
    pub head: usize,
    pub data: Vec<T>,
}

impl<T> Stack<T> {
    /// Create an empty Stack object.
    pub fn default() -> Stack<T> {
        Stack {
            size: 0,
            head: 0,
            data: Vec::new(),
        }
    }

    /// Create a stack with a given size
    ///
    /// # Arguments
    ///
    /// *'size' The size of the stack to be created.
    pub fn new(size: usize) -> Stack<T> {
        Stack {
            size: size,
            head: 0,
            data: Vec::with_capacity(size),
        }
    }

    /// Push an item onto the stack, if there's space
    ///
    /// # Arguments
    ///
    /// *'item' The item to be pushed onto the stack.
    pub fn push(&mut self, item: T) -> Result<(), Error> {
        if self.head < self.size {
            self.head += 1;
            self.data.push(item);
            Ok(())
        } else {
            Err(Error::from(ErrorKind::Other))
        }
    }

    /// Pop an item off of the stack
    pub fn pop(&mut self) -> Result<T, Error> {
        if self.head > 0 {
            match self.data.pop() {
                Some(s) => {
                    self.head -= 1;
                    return Ok(s);
                }
                _ => return Err(Error::from(ErrorKind::Other)),
            }
        } else {
            Err(Error::from(ErrorKind::Other))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_constructor() {
        let empty: Stack<u8> = Stack::default();
        assert_eq!(0, empty.size);
        assert_eq!(0, empty.head);
        assert_eq!(0, empty.data.len());
    }

    #[test]
    fn size_constructor() {
        let size: usize = 10;
        let empty: Stack<u8> = Stack::new(size);
        assert_eq!(size, empty.size);
        assert_eq!(0, empty.head);
        assert_eq!(size, empty.data.capacity());
        assert_eq!(0, empty.data.len());
    }

    #[test]
    fn push_empty() {
        let mut empty: Stack<u8> = Stack::default();
        let result = empty.push(1);
        assert!(result.is_err());
    }

    #[test]
    fn pop_empty() {
        let mut empty: Stack<u8> = Stack::default();
        let result = empty.pop();
        assert!(result.is_err());
    }

    #[test]
    fn push_and_pop() {
        let mut empty: Stack<u8> = Stack::new(1);
        let result = empty.push(1);
        assert!(!result.is_err());

        let result = empty.push(2);
        assert!(result.is_err());

        let result = empty.pop();
        assert!(!result.is_err());
        assert_eq!(1, result.unwrap());
    }
}
