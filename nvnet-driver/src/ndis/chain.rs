use crate::windows::km::ndis::nbl::NET_BUFFER_LIST;

pub struct NblChain;

impl NblChain {
    pub unsafe fn last(nbl_chain: *mut NET_BUFFER_LIST) -> *mut NET_BUFFER_LIST {
        let mut last = nbl_chain;
        loop {
            let next = (*last).next;
            if next.is_null() {
                return last;
            }
            last = next;
        }
    }
}
