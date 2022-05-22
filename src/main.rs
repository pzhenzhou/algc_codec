use algc_codec::codec::{Codec, Triple};
use algc_codec::{algc_encode, codec, get_triple_value};
use clap::Parser;
use codec::TripleValues;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct CmdArgs {
    #[clap(short, long)]
    input_string: String,
    #[clap(short, long)]
    search_buffer_size: Option<usize>,
}

fn main() {
    let args = CmdArgs::parse();
    let raw_string: String = args.input_string;
    println!("ALG-C Receive Input String = {:?}", raw_string);
    if raw_string.is_empty() {
        println!("The compressed string must not be empty");
        return;
    }
    let encode_triple: Vec<Triple> = if let Some(buffer_size) = args.search_buffer_size {
        algc_encode!(raw_string.clone(), Some(buffer_size), |triple: Triple| {
            triple
        })
    } else {
        algc_encode!(raw_string.clone(), None, |triple: Triple| triple)
    };
    println!("encode_triple complete={:#?}", encode_triple);
    let triple_vec = TripleValues::TripleVec(encode_triple);
    let decode_string = Codec::decode(triple_vec, |triple_value| {
        get_triple_value!(triple_value, TripleVec)
    });
    assert_eq!(raw_string, decode_string);
}
