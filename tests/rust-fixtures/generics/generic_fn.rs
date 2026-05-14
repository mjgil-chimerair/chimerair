// Test fixture: generic_fn.rs - generic instantiation testing
// @verify chimera-rust-cache
// @expected ch_status SUCCESS

pub fn identity<T: Clone>(x: T) -> T {
    x.clone()
}

pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

pub fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
}
