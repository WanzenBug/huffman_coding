use std::io::{Read, Write, Result};

pub struct StatsReader<R> where R: Read {
    inner: R,
    read_bytes: usize,
}

pub struct StatsWriter<W> where W: Write {
    inner: W,
    written_bytes: usize,
}

pub struct Stats {
    pub processed: usize,
}

impl<R> StatsReader<R> where R: Read {
    pub fn new(reader: R) -> Self {
        StatsReader {
            inner: reader,
            read_bytes: 0
        }
    }

    pub fn get_stats(&self) -> Stats {
        Stats {
            processed: self.read_bytes
        }
    }
}

impl<W> StatsWriter<W> where W: Write {
    pub fn new(writer: W) -> Self {
        StatsWriter {
            inner: writer,
            written_bytes: 0,
        }
    }

    pub fn get_stats(&self) -> Stats {
        Stats {
            processed: self.written_bytes,
        }
    }
}

impl<R> Read for StatsReader<R> where R: Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.inner.read(buf)?;
        self.read_bytes += n;
        Ok(n)
    }
}

impl<W> Write for StatsWriter<W> where W: Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = self.inner.write(buf)?;
        self.written_bytes += n;
        Ok(n)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}