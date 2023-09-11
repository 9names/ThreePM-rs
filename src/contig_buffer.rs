use core::{fmt, slice::Chunks};

const BUFF_SZ: usize = 1024;
const CHUNK_SZ: usize = 512;
#[derive(Debug)]
pub(crate) struct Buffer {
    pub mp3_byte_buffer: [u8; BUFF_SZ],
    pub buff_start: usize,
    pub buff_end: usize,
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "start:{} end:{} used:{} avail:{}",
            self.buff_start,
            self.buff_end,
            self.used(),
            self.available()
        )
    }
}

#[allow(unused)]
impl Buffer {
    pub const fn new() -> Self {
        Self {
            mp3_byte_buffer: [0u8; BUFF_SZ],
            buff_start: 0,
            buff_end: 0,
        }
    }

    /// How much data is stored in the buffer
    pub fn used(&self) -> usize {
        self.buff_end - self.buff_start
    }

    /// How much free space is in the buffer
    pub fn available(&self) -> usize {
        BUFF_SZ - self.used()
    }

    /// How much contiguous free space there is at the end of the buffer
    pub fn tail_free(&self) -> usize {
        BUFF_SZ - self.buff_end
    }

    /// Shuffle all bytes along so that start of buffer == start of data
    pub fn remove_unused(&mut self) {
        if self.buff_start != 0 {
            self.mp3_byte_buffer
                .copy_within((self.buff_start)..(self.buff_end), 0);
            let used = self.used();
            self.buff_start = 0;
            self.buff_end = used;
        }
    }

    /// Using the provided iterator, load more data into the buffer
    pub fn load_more(&mut self, loader: &mut Chunks<'_, u8>) -> bool {
        let mut loaded_some = false;
        // if we need more contiguous space, shuffle the data to the start
        if self.tail_free() < CHUNK_SZ {
            self.remove_unused();
        }
        while self.available() >= CHUNK_SZ {
            if let Some(d) = loader.next() {
                let newend = self.buff_end + d.len();
                self.mp3_byte_buffer[(self.buff_end)..(newend)].copy_from_slice(d);
                self.buff_end = newend;
                loaded_some = true;
            } else {
                return loaded_some;
            }
        }
        loaded_some
    }

    /// Using the provided slice, load more data into the buffer.
    /// Returns the number of bytes consumed
    pub fn load_slice(&mut self, data: &[u8]) -> usize {
        if self.tail_free() < data.len() {
            self.remove_unused();
        }
        let loadsize = usize::min(self.tail_free(), data.len());
        let newend = self.buff_end + loadsize;
        self.mp3_byte_buffer[(self.buff_end)..(newend)].copy_from_slice(data);
        self.buff_end = newend;

        loadsize
    }

    /// Increment our "start pointer". use this as you consume slices from the start
    pub fn increment_start(&mut self, increment: usize) {
        self.buff_start += increment;
    }

    /// Return a slice over the remaining data in the buffer
    pub fn borrow_slice(&self) -> &[u8] {
        &self.mp3_byte_buffer[self.buff_start..self.buff_end]
    }

    /// Return a slice over the remaining data in the buffer and update the indexes
    /// this should be safe, since the &mut is active as long as the slice is borrowed
    pub fn take_slice(&mut self) -> &[u8] {
        let start = self.buff_start;
        let end = self.buff_end;
        self.buff_start = 0;
        self.buff_end = 0;

        &self.mp3_byte_buffer[start..end]
    }

    /// Return a slice over some of the data and update the indexes
    /// this should be safe, since the &mut is active as long as the slice is borrowed
    /// if you request more data than is present, you get an error
    pub fn take_subslice(&mut self, slice_size: usize) -> Result<&[u8], ()> {
        if slice_size <= self.used() {
            let start = self.buff_start;
            let end = start + slice_size;
            // update the start of data index to be beyond what we returned
            self.buff_start = end;

            Ok(&self.mp3_byte_buffer[start..end])
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_good() {
        let buffer = Buffer::new();
        assert_eq!(buffer.used(), 0);
        assert_eq!(buffer.available(), BUFF_SZ);
        assert_eq!(buffer.tail_free(), BUFF_SZ);
        let data = buffer.borrow_slice();
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn add_data() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3]);
        assert_eq!(buffer.used(), 4);
        assert_eq!(buffer.available(), BUFF_SZ - 4);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 4);
    }

    #[test]
    fn add_multiple_data() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3]);
        buffer.load_slice(&[4, 5, 6, 7]);
        let data = buffer.take_slice();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0, 1, 2, 3, 4, 5, 6, 7]);
        let _ = data;
        assert_eq!(buffer.used(), 0);
        assert_eq!(buffer.available(), BUFF_SZ);
        assert_eq!(buffer.tail_free(), BUFF_SZ);
    }

    #[test]
    fn add_multiple_data_chunks() {
        let mut buffer = Buffer::new();
        // let data = &[0,1,2,3,4,5,6,7];
        let data = &[42; BUFF_SZ * 2];
        let iter = data.chunks(CHUNK_SZ);
        assert_eq!(iter.count(), 4);
        let mut iter = data.chunks(CHUNK_SZ);
        buffer.load_more(&mut iter);
        assert_eq!(buffer.used(), BUFF_SZ);
        assert_eq!(buffer.available(), 0);
        assert_eq!(buffer.tail_free(), 0);
        assert_eq!(iter.count(), 2);
    }

    #[test]
    fn fill_buffer() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[42; BUFF_SZ]);
        let data = buffer.take_slice();
        assert_eq!(data.len(), BUFF_SZ);
        assert_eq!(data, &[42; BUFF_SZ]);
        let _ = data;
        assert_eq!(buffer.used(), 0);
        assert_eq!(buffer.available(), BUFF_SZ);
        assert_eq!(buffer.tail_free(), BUFF_SZ);
    }

    #[test]
    fn borrow_data() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3]);
        let data = buffer.borrow_slice();
        assert_eq!(data.len(), 4);
        assert_eq!(data, &[0, 1, 2, 3]);
        let _ = data;
        assert_eq!(buffer.used(), 4);
        assert_eq!(buffer.available(), BUFF_SZ - 4);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 4);
        buffer.increment_start(4);
        assert_eq!(buffer.used(), 0);
        assert_eq!(buffer.available(), BUFF_SZ);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 4);
    }

    #[test]
    fn take_data() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3]);
        let data = buffer.take_slice();
        assert_eq!(data.len(), 4);
        assert_eq!(data, &[0, 1, 2, 3]);
        let _ = data;
        assert_eq!(buffer.used(), 0);
        assert_eq!(buffer.available(), BUFF_SZ);
        assert_eq!(buffer.tail_free(), BUFF_SZ);
    }

    #[test]
    fn take_some() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3]);
        let data = buffer.borrow_slice();
        assert_eq!(data.len(), 4);
        assert_eq!(data, &[0, 1, 2, 3]);
        let _ = data;
        assert_eq!(buffer.used(), 4);
        assert_eq!(buffer.available(), BUFF_SZ - 4);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 4);
        buffer.increment_start(4);
        assert_eq!(buffer.used(), 0);
        assert_eq!(buffer.available(), BUFF_SZ);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 4);
    }

    #[test]
    fn take_less_than_whole() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3, 4, 5, 6, 7]);
        let data = buffer.borrow_slice();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0, 1, 2, 3, 4, 5, 6, 7]);
        let _ = data;
        assert_eq!(buffer.used(), 8);
        assert_eq!(buffer.available(), BUFF_SZ - 8);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 8);
        buffer.increment_start(4);
        assert_eq!(buffer.used(), 4);
        assert_eq!(buffer.available(), BUFF_SZ - 4);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 8);
    }

    #[test]
    fn take_less_than_whole_then_shuffle() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[0, 1, 2, 3, 4, 5, 6, 7]);
        let data = buffer.borrow_slice();
        assert_eq!(data.len(), 8);
        assert_eq!(data, &[0, 1, 2, 3, 4, 5, 6, 7]);
        let _ = data;
        assert_eq!(buffer.used(), 8);
        assert_eq!(buffer.available(), BUFF_SZ - 8);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 8);
        buffer.increment_start(4);
        buffer.remove_unused();
        assert_eq!(buffer.used(), 4);
        assert_eq!(buffer.available(), BUFF_SZ - 4);
        assert_eq!(buffer.tail_free(), BUFF_SZ - 4);
    }

    #[test]
    fn fill_buffer_then_take_then_add() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[42; BUFF_SZ]);
        let data = buffer.take_subslice(4).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(data, &[42; 4]);
        let _ = data;
        assert_eq!(buffer.used(), BUFF_SZ - 4);
        assert_eq!(buffer.available(), 4);
        assert_eq!(buffer.tail_free(), 0);
        buffer.remove_unused();
        assert_eq!(buffer.used(), BUFF_SZ - 4);
        assert_eq!(buffer.available(), 4);
        assert_eq!(buffer.tail_free(), 4);
    }

    #[test]
    fn fill_buffer_then_take_then_add_some_more() {
        let mut buffer = Buffer::new();
        buffer.load_slice(&[42; BUFF_SZ]);
        // buffer is now full, take 4
        let data = buffer.take_subslice(4).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(data, &[42; 4]);
        let _ = data;
        // we should have 4 space free
        assert_eq!(buffer.used(), BUFF_SZ - 4);
        assert_eq!(buffer.available(), 4);
        // but none of it is at the end
        assert_eq!(buffer.tail_free(), 0);
        // take another 4
        let data = buffer.take_subslice(4).unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(data, &[42; 4]);
        let _ = data;
        // we should have 8 space free
        assert_eq!(buffer.used(), BUFF_SZ - 8);
        assert_eq!(buffer.available(), 8);
        // but none of it is at the end
        assert_eq!(buffer.tail_free(), 0);
        // add 4 bytes to our buffer
        // this should cause a shuffle of bytes to the front
        buffer.load_slice(&[69; 4]);
        // we should have 4 space free
        assert_eq!(buffer.used(), BUFF_SZ - 4);
        assert_eq!(buffer.available(), 4);
        // and all of it at the end
        assert_eq!(buffer.tail_free(), 4);
        let data = buffer.borrow_slice();
        assert_eq!(data.len(), BUFF_SZ - 4);
        // the first chunk of the data should be all 42s
        assert_eq!(&data[0..BUFF_SZ - 8], &[42; BUFF_SZ - 8]);
        // the last 4 bytes should be 69s
        assert_eq!(&data[BUFF_SZ - 8..BUFF_SZ - 4], &[69; 4]);
    }
}
