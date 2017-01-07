extern crate huffman_coding;

#[cfg(feature = "bin")]
extern crate clap;

#[cfg(feature = "bin")]
mod util;


#[cfg(feature = "bin")]
fn main() {
    use std::fs::File;
    use std::io::{copy, BufReader, BufWriter, Write, Read, Cursor};
    use huffman_coding::{HuffmanWriter, HuffmanTree};

    let matches = clap::App::new("Huffman encoder")
        .version("0.1")
        .author("Moritz Wanzenböck <moritz.wanzenboeck@gmail.com>")
        .about("Compresses files using pure Huffman coding")
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
        let mut vec = Vec::new();
        read.read_to_end(&mut vec).expect("Error reading input");
        let tree = HuffmanTree::from_data(&vec[..]);
        let table = tree.to_table();
        write.write_all(&table).expect("Could not write encoding table!");

        let mut encoder = HuffmanWriter::new(&mut write, &tree);
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

#[cfg(not(feature = "bin"))]
fn main() {
    println!("Feature not enabled");
    std::process::exit(1);
}