extern crate bitstream;
extern crate bit_vec;

use std::io::{Write, Read};
use std::io::Result as IOResult;
use std::io::Error as IOError;
use std::io::ErrorKind as IOErrorKind;
use std::cmp;
use std::collections::{HashMap, BinaryHeap};
use bit_vec::BitVec;

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
        let comb = heap.pop().unwrap();
        let comb2 = heap.pop().unwrap();
        HuffmanTree::Node(Box::new(comb), Box::new(comb2))
    }

    pub fn new(data: &[u8]) -> Self {
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

    fn get_byte_prob(&self, byte: u8) -> Option<u8> {
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

pub struct HuffmanWriter<W> where W: Write {
    inner: bitstream::BitWriter<W>,
    table: HashMap<u8, BitVec>,
}

impl<W> HuffmanWriter<W> where W: Write {
    pub fn new(mut writer: W, tree: &HuffmanTree) -> IOResult<Self> {
        for i in 0..256 {
            let prob = tree.get_byte_prob(i as u8).unwrap_or(0);
            writer.write_all(&[prob])?;
        }

        Ok(HuffmanWriter {
            inner: bitstream::BitWriter::new(writer),
            table: tree.to_lookup_table()
        })
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

impl<R> HuffmanReader<R> where R: Read {
    pub fn new(mut reader: R) -> IOResult<Self> {
        let mut table: [u8; 256] = [0; 256];
        reader.read_exact(&mut table[..])?;
        let tree = HuffmanTree::from_table(&table);
        Ok(HuffmanReader {
            inner: bitstream::BitReader::new(reader),
            tree: tree,
        })
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
        let tree = HuffmanTree::new(&vec[..]);
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
        let tree = HuffmanTree::new(&pseudo_data[..]);

        let mut vec = Vec::new();
        {
            let mut writer = HuffmanWriter::new(&mut vec, &tree).expect("Writer is not ok!");
            assert!(writer.write(&[0, 0, 1, 1, 2, 2, 2, 2]).is_ok())
        }
        assert_eq!(&vec[..3], &[255, 127, 255]);
        assert_eq!(&vec[256..], &[175, 0 , 4]);
    }

    #[test]
    fn test_reader() {
        let mut pseudo_input = vec![0; 259];
        pseudo_input[0] = 255;
        pseudo_input[1] = 128;
        pseudo_input[2] = 255;
        pseudo_input[256] = 175;
        pseudo_input[257] = 0;
        pseudo_input[258] = 4;
        use std::io::Cursor;
        let mut buf = vec![0; 8];
        let mut read = Cursor::new(pseudo_input);
        let mut read = HuffmanReader::new(&mut read).expect("Reader is not ok!");
        use std::io::Read;
        assert!(read.read_exact(&mut buf[..]).is_ok());
        assert_eq!(&buf[..], &[0, 0, 1, 1, 2, 2, 2, 2]);
        let read_end = read.read(&mut buf[..]);
        assert!(read_end.is_ok());
        assert_eq!(read_end.unwrap(), 0);
    }
}

