c_type!(
    pub struct NET_FRAGMENT {
        _bits: u64,
    }
);

impl NET_FRAGMENT {
    const VALID_LENGTH_OFFSET: u64 = 0;
    const VALID_LENGTH_WIDTH: u64 = 26;
    const VALID_LENGTH_MASK: u64 = u64::MAX >> (64 - Self::VALID_LENGTH_WIDTH);

    const CAPACITY_OFFSET: u64 = Self::VALID_LENGTH_OFFSET + 26;
    const CAPACITY_WIDTH: u64 = 26;
    const CAPACITY_MASK: u64 = u64::MAX >> (64 - Self::CAPACITY_WIDTH);

    const OFFSET_OFFSET: u64 = Self::CAPACITY_OFFSET + 26;
    const OFFSET_WIDTH: u64 = 10;
    const OFFSET_MASK: u64 = u64::MAX >> (64 - Self::OFFSET_WIDTH);

    pub fn valid_length(&mut self) -> u64 {
        (self._bits >> Self::VALID_LENGTH_OFFSET) & Self::VALID_LENGTH_MASK
    }

    pub fn set_valid_length(&mut self, valid_length: u64) {
        self._bits &= u64::MAX << (Self::VALID_LENGTH_OFFSET + Self::VALID_LENGTH_WIDTH);
        self._bits |= (valid_length & Self::VALID_LENGTH_MASK) << Self::VALID_LENGTH_OFFSET
    }

    pub fn capacity(&self) -> u64 {
        (self._bits >> Self::CAPACITY_OFFSET) & Self::CAPACITY_MASK
    }

    pub fn offset(&mut self) -> u64 {
        (self._bits >> Self::OFFSET_OFFSET) & Self::OFFSET_MASK
    }

    pub fn set_offset(&mut self, offset: u64) {
        self._bits &= (u64::MAX << (Self::OFFSET_OFFSET + Self::OFFSET_WIDTH))
            | (u64::MAX >> (64 - Self::OFFSET_OFFSET));
        self._bits |= (offset & Self::OFFSET_MASK) << Self::OFFSET_OFFSET
    }
}
