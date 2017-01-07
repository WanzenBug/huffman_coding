extern crate huffman;
extern crate clap;

mod util;

use std::fs::File;
use std::io::{copy, BufReader, BufWriter, Write, Read, Cursor};
use huffman::{HuffmanWriter, HuffmanTree};
use clap::{App, Arg};

fn main() {
    let matches = App::new("Huffman encoder")
        .version("0.1")
        .author("Moritz Wanzenböck <moritz.wanzenboeck@gmail.com>")
        .about("Compresses files using pure Huffman coding")
        .arg(Arg::with_name("INPUT")
            .required(true)
            .help("Sets the input file to use")
            .index(1))
        .arg(Arg::with_name("OUTPUT")
            .required(true)
            .help("Sets the output file to use")
            .index(2))
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("Sets verbose output"))
        .get_matches();

    let infile = matches.value_of("INPUT").and_then(|file| File::open(file).ok()).expect("Input: No such file");
    let outfile = matches.value_of("OUTPUT").and_then(|file| File::create(file).ok()).expect("Output: Could not create file");
    let mut read = util::StatsReader::new(BufReader::new(infile));
    let mut write = util::StatsWriter::new(BufWriter::new(outfile));
    {
        let mut vec = Vec::new();
        read.read_to_end(&mut vec).expect("Error reading input");
        let tree = HuffmanTree::new(&vec[..]);
        let mut encoder = HuffmanWriter::new(&mut write, &tree).expect("Could not write huffman table");
        let mut read_vec = Cursor::new(vec);
        copy(&mut read_vec, &mut encoder).expect("Something went wrong while decoding");
    }
    if matches.is_present("verbose") {
        write.flush().expect("Flush failed");
        let written = write.get_stats().processed;
        let read = read.get_stats().processed;

        println!("Read:    {} bytes", read);
        println!("Written: {} bytes", written);
    }
}