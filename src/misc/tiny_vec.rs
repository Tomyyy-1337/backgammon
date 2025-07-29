use std::{mem::MaybeUninit, ops::Deref};

#[derive(Debug, Clone, Copy)]
pub struct TinyVec<T, const N: usize> 
where
    T: Copy,
{
    buf: [MaybeUninit<T>; N],
    len: u8,
}

impl<T, const N: usize> TinyVec<T, N> 
where
    T: Copy
{
    pub const fn new() -> Self {
        const { assert!(N <= 255, "TinyVec supports up to 255 elements") }
        // SAFETY: MaybeUninit array is uninitialized — that's fine
        let buf = unsafe { MaybeUninit::uninit().assume_init() };
        TinyVec { buf, len: 0 }
    }

    pub const fn from_raw(buf: [MaybeUninit<T>; N], len: u8) -> Self {
        TinyVec { buf, len }
    }

    pub const fn push(&mut self, val: T) {
        self.buf[self.len as usize].write(val);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None; }
        self.len -= 1;
        // SAFETY: previously pushed, so initialized
        Some(unsafe { self.buf[self.len as usize].assume_init_read() })
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.len as usize {
            // SAFETY: the slot was initialized
            Some(unsafe { &*self.buf[idx].as_ptr() })
        } else { None }
    }
    
    pub fn len(&self) -> u8 {
        self.len
    }
}

impl<T: Copy, const N: usize> Deref for TinyVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY: Only the first `self.len` elements are initialized
        unsafe {
            std::slice::from_raw_parts(self.buf.as_ptr() as *const T, self.len as usize)
        }
    }
}