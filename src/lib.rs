//! **huffman_coding** is a small library for reading and writing huffman encoded data
//!
//! There are only 3 things you need to know

extern crate bitstream;
extern crate bit_vec;

use std::io::{Write, Read};
use std::io::Result as IOResult;
use std::io::Error as IOError;
use std::io::ErrorKind as IOErrorKind;
use std::cmp;
use std::collections::{HashMap, BinaryHeap};
use bit_vec::BitVec;

/// *HuffmanTree* is a simple tree structure used convert encoded words to decoded words and
/// vice versa.
///
/// Each leaf of the tree represents a single code word. Their probability is saved as single byte
/// where 255 represents the highest probability, and 0 means the value does not appear.
///
/// You most likely don't want to construct this tree yourself, so look for the 2 methods
/// for constructing the tree for you.
///
/// # Examples
/// ```
/// extern crate huffman_coding;
///
/// let fake_data = vec![1, 1, 0, 0, 2];
/// let tree = huffman_coding::HuffmanTree::from_data(&fake_data[..]);
/// let probability = tree.get_byte_prob(1);
/// assert!(probability.is_some());
/// assert_eq!(probability.unwrap(), 255);
/// ```
#[derive(Eq, Debug)]
pub enum HuffmanTree {
    Leaf(u8, u8),
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let own_prob = self.get_probability();
        let other_prob = other.get_probability();

        // We want to use the std heap, which is a max heap. However, we want to have
        // the minimum probability on top
        if own_prob > other_prob {
            cmp::Ordering::Less
        } else if own_prob == other_prob {
            cmp::Ordering::Equal
        } else {
            cmp::Ordering::Greater
        }
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HuffmanTree {
    fn eq(&self, other: &HuffmanTree) -> bool {
        match (self, other) {
            (&HuffmanTree::Leaf(ref x1, ref prob1), &HuffmanTree::Leaf(ref x2, ref prob2)) => {
                x1 == x2 && prob1 == prob2
            },
            (&HuffmanTree::Node(ref zero1, ref one1), &HuffmanTree::Node(ref zero2, ref one2)) => {
                zero1 == zero2 && one1 == one2
            },
            _ => false
        }
    }
}

impl HuffmanTree {
    /// Method to read the probability of all 256 possible u8 values from a slice containing 256
    /// elements.
    ///
    /// This method can be used to construct a new tree from a list of probabilities. The first
    /// element in the slice will be interpreted as the probability of the `0` value appearing, the
    /// second as the probability of the `1` value, etc.
    ///
    /// # Examples
    /// ```
    /// extern crate huffman_coding;
    ///
    /// let mut table_data: [u8; 256] = [0; 256];
    /// table_data[0] = 255;
    /// table_data[1] = 128;
    /// table_data[2] = 128;
    /// let tree = huffman_coding::HuffmanTree::from_table(&table_data[..]);
    ///
    /// let test_query = tree.get_byte_prob(1);
    /// assert!(test_query.is_some());
    /// assert_eq!(test_query.unwrap(), 128);
    /// ```
    /// # Panics
    /// If data contains less than 256 elements
    pub fn from_table(data: &[u8]) -> Self {
        let mut heap: BinaryHeap<_> = data
            .iter()
            .enumerate()
            .filter(|x| *x.1 > 0)
            .map(|x| HuffmanTree::Leaf(x.0 as u8, *x.1))
            .collect();

        while heap.len() > 2 {
            let a = heap.pop().unwrap();
            let b = heap.pop().unwrap();
            let insert = if a < b {
                HuffmanTree::Node(Box::new(a), Box::new(b))
            } else {
                HuffmanTree::Node(Box::new(b), Box::new(a))
            };
            heap.push(insert);
        }
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        HuffmanTree::Node(Box::new(a), Box::new(b))
    }

    /// Reads all of data and constructs a huffman tree according to the provided sample data
    ///
    /// # Examples
    /// ```
    /// extern crate huffman_coding;
    /// let pseudo_data = vec![0, 0, 1, 2, 2];
    /// let tree = huffman_coding::HuffmanTree::from_data(&pseudo_data[..]);
    ///
    /// let test_query = tree.get_byte_prob(0);
    /// assert!(test_query.is_some());
    /// assert_eq!(test_query.unwrap(), 255);
    /// ```
    pub fn from_data(data: &[u8]) -> Self {
        let mut probability: [usize; 256] = [0; 256];
        let mut max = 0;
        for item in data {
            probability[*item as usize] += 1;

            if probability[*item as usize] > max {
                max = probability[*item as usize];
            }
        }

        let norm = HuffmanTree::normalize(&probability, max);
        HuffmanTree::from_table(&norm[..])
    }

    /// Convert an existing huffman tree into an array where each element represents the probability
    /// of the index byte to appear according to the huffman tree
    ///
    /// This can be used to transmit the encoding scheme via byte buffer
    pub fn to_table(&self) -> [u8; 256] {
        let mut table: [u8; 256] = [0; 256];
        for i in 0..256 {
            table[i] = self.get_byte_prob(i as u8).unwrap_or(0);
        }
        table
    }

    /// Return the probability of the given byte to appear according to the tree
    ///
    /// If this returns None, then the byte should not appear according to the huffman tree
    /// If this returns Some, it will be between 255 meaning highest probability, and 1, meaning
    /// lowest probability
    pub fn get_byte_prob(&self, byte: u8) -> Option<u8> {
        match self {
            &HuffmanTree::Leaf(item, prob) if item == byte => Some(prob),
            &HuffmanTree::Node(ref zero, ref one) => {
                zero.get_byte_prob(byte).or(one.get_byte_prob(byte))
            },
            _ => None
        }
    }

    fn normalize(data: &[usize], max_elem: usize) -> [u8; 256] {
        let mut normalized_data: [u8; 256] = [0; 256];

        for i in 0..data.len() {
            if data[i] > 0 {
                normalized_data[i] = cmp::max((data[i] * 255 / max_elem) as u8, 1);
            }
        }
        normalized_data
    }

    fn get_probability(&self) -> u16 {
        match self {
            &HuffmanTree::Leaf(_, prob) => prob as u16,
            &HuffmanTree::Node(ref zero, ref one) => {
                zero.get_probability() + one.get_probability()
            }
        }
    }

    fn to_lookup_table(&self) -> HashMap<u8, BitVec> {
        let mut table = HashMap::new();
        self.to_lookup_table_inner(&mut table, BitVec::new());
        table
    }

    fn to_lookup_table_inner(&self, data: &mut HashMap<u8, BitVec>, prev: BitVec) {
        match self {
            &HuffmanTree::Leaf(ref elem, _) => {
                data.insert(*elem, prev);
            },
            &HuffmanTree::Node(ref zero, ref one) => {
                let mut zero_branch = prev.clone();
                zero_branch.push(false);
                zero.to_lookup_table_inner(data, zero_branch);
                let mut one_branch = prev;
                one_branch.push(true);
                one.to_lookup_table_inner(data, one_branch);
            }
        }
    }
}

/// *HuffmanWriter* is a Write implementation that writes encoded words to the
/// inner writer.
///
/// # Examples
///
/// ```
/// extern crate huffman_coding;
/// let pseudo_data = vec![0, 0, 1, 2, 2];
/// let tree = huffman_coding::HuffmanTree::from_data(&pseudo_data[..]);
///
/// let mut output = Vec::new();
/// {
///     use std::io::Write;
///     let mut writer = huffman_coding::HuffmanWriter::new(&mut output, &tree);
///     assert!(writer.write(&[2, 2, 0, 0, 1]).is_ok());
/// }
/// assert_eq!(&output[..], [43, 8]);
/// ```
pub struct HuffmanWriter<W> where W: Write {
    inner: bitstream::BitWriter<W>,
    table: HashMap<u8, BitVec>,
}

impl<W> HuffmanWriter<W> where W: Write {
    /// Construct a new HuffmanWriter using the provided HuffmanTree
    pub fn new(writer: W, tree: &HuffmanTree) -> Self {
        HuffmanWriter {
            inner: bitstream::BitWriter::new(writer),
            table: tree.to_lookup_table()
        }
    }
}

impl<W> Write for HuffmanWriter<W> where W: Write {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        for item in buf {
            let bits = self.table.get(item).ok_or(IOError::from(IOErrorKind::InvalidData))?;
            for bit in bits {
                self.inner.write_bit(bit)?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> IOResult<()> {
        Ok(())
    }
}

pub struct HuffmanReader<R> where R: Read {
    inner: bitstream::BitReader<R>,
    tree: HuffmanTree,
}

/// *HuffmanReader* is a Read implementation that can read encoded words from the inner reader
///
/// # Examples
/// ```
/// extern crate huffman_coding;
/// let pseudo_data = vec![0, 0, 1, 2, 2];
/// let tree = huffman_coding::HuffmanTree::from_data(&pseudo_data[..]);
///
/// use std::io::{Read, Cursor};
/// let cursor = Cursor::new([43, 8]);
/// let mut buffer: [u8; 5] = [0; 5];
///
/// let mut reader = huffman_coding::HuffmanReader::new(cursor, tree);
/// assert!(reader.read_exact(&mut buffer[..]).is_ok());
/// assert_eq!(&buffer[..], &[2, 2, 0, 0, 1]);
/// ```
impl<R> HuffmanReader<R> where R: Read {
    /// Construct a new reader, using the provided HuffmanTree for decoding
    pub fn new(reader: R, tree: HuffmanTree) -> Self {
        HuffmanReader {
            inner: bitstream::BitReader::new(reader),
            tree: tree,
        }
    }
}

impl<R> Read for HuffmanReader<R> where R: Read {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        let mut pos = 0;
        let mut state = &self.tree;
        while pos < buf.len() {
            let bit_opt = self.inner.read_bit()?;
            if let Some(bit) = bit_opt {
                match state {
                    &HuffmanTree::Leaf(x, _) => {
                        buf[pos] = x;
                        pos += 1;
                        state = &self.tree;
                    },
                    &HuffmanTree::Node(ref zero, ref one) => {
                        state = if bit { one } else { zero };
                        if let &HuffmanTree::Leaf(x, _) = state {
                            buf[pos] = x;
                            pos += 1;
                            state = &self.tree;
                        }
                    }
                }
            } else {
                if &self.tree != state {
                    return Err(IOError::from(IOErrorKind::InvalidData))
                } else {
                    break;
                }
            }
        }
        Ok((pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bit_vec::BitVec;

    #[test]
    fn test_tree_builder() {
        let vec = vec![1, 2, 3, 1, 1, 2];
        let tree = HuffmanTree::from_data(&vec[..]);
        let table = tree.to_lookup_table();

        use std::iter::FromIterator;
        assert_eq!(table[&1u8], BitVec::from_iter(vec![false].into_iter()));
        assert_eq!(table[&2u8], BitVec::from_iter(vec![true, false].into_iter()));
        assert_eq!(table[&3u8], BitVec::from_iter(vec![true, true].into_iter()));
    }

    #[test]
    fn test_writer() {
        use std::io::Write;
        let pseudo_data = vec![0, 0, 1, 2, 2];
        let tree = HuffmanTree::from_data(&pseudo_data[..]);

        let mut vec = Vec::new();
        {
            let mut writer = HuffmanWriter::new(&mut vec, &tree);
            assert!(writer.write(&[0, 0, 1, 1, 2, 2, 2, 2]).is_ok())
        }
        assert_eq!(&vec[..], &[175, 0 , 4]);
    }

    #[test]
    fn test_reader() {
        let mut table: [u8; 256] = [0; 256];
        table[0] = 255;
        table[1] = 128;
        table[2] = 255;
        let tree = HuffmanTree::from_table(&table[..]);

        let mut input: [u8; 3] = [0; 3];
        input[0] = 175;
        input[1] = 0;
        input[2] = 4;
        use std::io::Cursor;
        let mut buf = vec![0; 8];
        let mut read = HuffmanReader::new(Cursor::new(input), tree);
        use std::io::Read;
        assert!(read.read_exact(&mut buf[..]).is_ok());
        assert_eq!(&buf[..], &[0, 0, 1, 1, 2, 2, 2, 2]);
        let read_end = read.read(&mut buf[..]);
        assert!(read_end.is_ok());
        assert_eq!(read_end.unwrap(), 0);
    }
}

