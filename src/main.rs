use algc_codec::codec::Codec;
use clap::Parser;

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
    let codec = Codec::new(raw_string.clone());
    let encode_triple = if let Some(buffer_size) = args.search_buffer_size {
        codec.default_encode(Some(buffer_size))
    } else {
        codec.default_encode(None)
    };
    println!("encode_triple complete={:#?}", encode_triple);
    assert_eq!(raw_string, Codec::decode(&encode_triple));
}
