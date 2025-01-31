use std::{mem::MaybeUninit, ptr::NonNull};

const CHUNK_CAPACITY: usize = 16;

pub struct ChunkedVectorPool<T: Copy> {
    chunks: Vec<NonNull<Chunk<T>>>,
    bump: bumpalo::Bump,
}

/// A chunked vector that can takes its memory from a pool that can be shared between many vectors.
pub struct ChunkedVector<T: Copy> {
    data: Option<NonNull<Chunk<T>>>,
}

pub struct Chunk<T: Copy> {
    data: [MaybeUninit<T>; CHUNK_CAPACITY],
    used: usize,
    next: Option<NonNull<Chunk<T>>>,
}

impl<T: Copy> ChunkedVector<T> {
    pub fn new() -> Self {
        ChunkedVector { data: None }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub fn first_chunk(&self) -> Option<&Chunk<T>> {
        self.data.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn push(&mut self, value: T, pool: &mut ChunkedVectorPool<T>) {
        if let Some(mut chunk) = self.data {
            let chunk = unsafe { chunk.as_mut() };
            if chunk.used < CHUNK_CAPACITY {
                chunk.data[chunk.used] = MaybeUninit::new(value);
                chunk.used += 1;
                return;
            }
        }
        let new_chunk = unsafe { pool.alloc().as_mut() };
        new_chunk.data[0] = MaybeUninit::new(value);
        new_chunk.used = 1;
        new_chunk.next = self.data;
        self.data = Some(new_chunk.into());
    }

    pub fn clear(&mut self, pool: &mut ChunkedVectorPool<T>) {
        let mut current_opt = self.data;
        while let Some(current) = current_opt {
            let next = unsafe { current.as_ref().next };
            pool.dealloc(current);
            current_opt = next;
        }
    }
}

impl<T: Copy> Chunk<T> {
    pub fn get_slice(&self) -> &[T] {
        let slice = &self.data[..self.used];
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const T, slice.len()) }
    }

    pub fn next_chunk(&self) -> Option<&Chunk<T>> {
        self.next.map(|ptr| unsafe { ptr.as_ref() })
    }
}

impl<T: Copy> ChunkedVectorPool<T> {
    pub fn new() -> Self {
        ChunkedVectorPool {
            chunks: vec![],
            bump: bumpalo::Bump::new(),
        }
    }

    fn alloc(&mut self) -> NonNull<Chunk<T>> {
        if let Some(mut chunk) = self.chunks.pop() {
            unsafe {
                chunk.as_mut().used = 0;
            }
            chunk
        } else {
            self.bump
                .alloc::<Chunk<T>>(Chunk {
                    data: [MaybeUninit::uninit(); CHUNK_CAPACITY],
                    used: 0,
                    next: None,
                })
                .into()
        }
    }

    fn dealloc(&mut self, ptr: NonNull<Chunk<T>>) {
        self.chunks.push(ptr);
    }
}
