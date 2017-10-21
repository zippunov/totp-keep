use std::ptr;

#[inline]
pub fn copy_memory(src: &[u8], dst: &mut [u8]) {
    assert!(dst.len() >= src.len());
    unsafe {
        let srcp = src.as_ptr();
        let dstp = dst.as_mut_ptr();
        ptr::copy_nonoverlapping(srcp, dstp, src.len());
    }
}

#[inline]
pub fn zero(dst: &mut [u8]) {
    unsafe {
        ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
    }
}