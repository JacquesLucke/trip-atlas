use anyhow::Result;
use std::path::Path;

pub struct MemoryMappedRkyv<'a, Archive: rkyv::Portable> {
    _mmap: memmap2::Mmap,
    data: &'a Archive,
}

impl<'a, Archive: rkyv::Portable> std::ops::Deref for MemoryMappedRkyv<'a, Archive> {
    type Target = Archive;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

// Safety: This is safe for as long as the underlying file is not modified.
pub async unsafe fn load_memory_mapped_rkyv<'a, Archive: rkyv::Portable + 'a>(
    path: &Path,
) -> Result<MemoryMappedRkyv<'a, Archive>> {
    let file = std::fs::File::open(path)?;
    // Safety: This is safe for as long as the underlying file is not modified.
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let buffer: &[u8] = unsafe { std::slice::from_raw_parts(mmap.as_ptr(), mmap.len()) };
    let rkyv_data = unsafe { rkyv::access_unchecked::<Archive>(buffer) };
    Ok(MemoryMappedRkyv {
        _mmap: mmap,
        data: rkyv_data,
    })
}
