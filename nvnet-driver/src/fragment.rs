use core::{mem, ptr::NonNull, slice};

use crate::{
    debug::ResultExt,
    linked::LinkedIter,
    windows::{
        km::{
            ndis::nbl::NET_BUFFER,
            wdm::{MmGetMdlByteCount, MmGetSystemAddressForMdlSafe, MDL, MM_PAGE_PRIORITY},
            wsk::WSK_BUF,
        },
        shared::ntstatus::STATUS_INSUFFICIENT_RESOURCES,
        Result,
    },
};

#[derive(Clone)]
pub struct FragmentIter<'a> {
    iter: LinkedIter<'a, MDL>,
    skip: usize,
    take: usize,
}

impl<'a> FragmentIter<'a> {
    pub unsafe fn from_nb(nb: *const NET_BUFFER) -> Self {
        Self {
            iter: LinkedIter::new((*nb).current_mdl),
            skip: (*nb).current_mdl_offset as usize,
            take: (*nb).data_length as usize,
        }
    }

    pub unsafe fn from_wb(wb: *const WSK_BUF) -> Self {
        Self {
            iter: LinkedIter::new((*wb).Mdl),
            skip: (*wb).Offset as usize,
            take: (*wb).Length,
        }
    }

    // effectively equivalent to NdisGetDataBuffer
    pub unsafe fn split<'b, T>(self, uninit: *mut T) -> Result<(&'b mut T, Self)> {
        let mut len = mem::size_of_val_raw(uninit);
        let mut self_clone = self.clone();
        let iter = Self {
            skip: self.skip + len,
            take: self.skip - len,
            ..self
        };
        let next = match self_clone.next() {
            None => todo!(),
            Some(next) => {
                let next = next?;
                if next.len() >= len {
                    let val = &mut *next.as_mut_ptr().cast();
                    return Ok((val, iter));
                }
                next
            }
        };
        let mut buf = slice::from_raw_parts_mut(uninit.cast(), len);
        let (buf_l, buf_r) = buf.split_at_mut(next.len());
        buf_l.copy_from_slice(next);
        buf = buf_r;
        len -= next.len();
        for next in self_clone {
            let next = next?;
            if next.len() >= len {
                buf.copy_from_slice(&next[..len]);
                return Ok((&mut *uninit, iter));
            }
            let (buf_l, buf_r) = buf.split_at_mut(next.len());
            buf_l.copy_from_slice(next);
            buf = buf_r;
            len -= next.len();
        }
        todo!();
    }
}

impl<'a> Iterator for FragmentIter<'a> {
    type Item = Result<&'a mut [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.take == 0 {
            return None;
        }
        while let Some(mdl) = self.iter.next() {
            let len = unsafe { MmGetMdlByteCount(mdl) as usize };
            if len <= self.skip {
                self.skip -= len;
                continue;
            }
            let skip = mem::take(&mut self.skip);
            let ptr = unsafe {
                MmGetSystemAddressForMdlSafe(
                    mdl,
                    MM_PAGE_PRIORITY::LowPagePriority | MM_PAGE_PRIORITY::MdlMappingNoExecute,
                )
            };
            let res = NonNull::new(ptr)
                .ok_or(STATUS_INSUFFICIENT_RESOURCES)
                .context_exit("MmGetSystemAddressForMdlSafe");
            let res = match res {
                Err(err) => Err(err),
                Ok(_) => {
                    let slice = unsafe { slice::from_raw_parts_mut(ptr.cast(), len) };
                    let mut slice = &mut slice[skip..];
                    let len = slice.len();
                    if len <= self.take {
                        self.take -= len;
                    } else {
                        let take = mem::take(&mut self.take);
                        slice = &mut slice[..take];
                    }
                    Ok(slice)
                }
            };
            return Some(res);
        }
        None
    }
}
