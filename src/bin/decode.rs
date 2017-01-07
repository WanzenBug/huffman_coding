extern crate huffman_coding;

#[cfg(feature = "bin")]
extern crate clap;

#[cfg(feature = "bin")]
mod util;

#[cfg(feature = "bin")]
fn main() {
    use std::fs::File;
    use std::io::{copy, BufReader, BufWriter, Write, Read};
    use huffman_coding::{HuffmanReader, HuffmanTree};

    let matches = clap::App::new("Huffman decoder")
        .version("0.1")
        .author("Moritz Wanzenböck <moritz.wanzenboeck@gmail.com>")
        .about("Decompress files using pure Huffman coding")
        .arg(clap::Arg::with_name("INPUT")
            .required(true)
            .help("Sets the input file to use")
            .index(1))
        .arg(clap::Arg::with_name("OUTPUT")
            .required(true)
            .help("Sets the output file to use")
            .index(2))
        .arg(clap::Arg::with_name("verbose")
            .short("v")
            .help("Sets verbose output"))
        .get_matches();

    let infile = matches.value_of("INPUT").and_then(|file| File::open(file).ok()).expect("Input: No such file");
    let outfile = matches.value_of("OUTPUT").and_then(|file| File::create(file).ok()).expect("Output: Could not create file");
    let mut read = util::StatsReader::new(BufReader::new(infile));
    let mut write = util::StatsWriter::new(BufWriter::new(outfile));
    {
        let mut table: [u8; 256] = [0; 256];
        read.read_exact(&mut table).expect("Could not read encoding table");
        let tree = HuffmanTree::from_table(&table);
        let mut reader = HuffmanReader::new(&mut read, tree);
        copy(&mut reader, &mut write).expect("Something went wrong while encoding");
    }
    if matches.is_present("verbose") {
        write.flush().expect("Flush failed");
        let written = write.get_stats().processed;
        let read = read.get_stats().processed;

        println!("Read:    {} bytes", read);
        println!("Written: {} bytes", written);
    }
}

#[cfg(not(feature = "bin"))]
fn main() {
    println!("Feature not enabled");
    std::process::exit(1);
}