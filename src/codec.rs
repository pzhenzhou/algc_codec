use crate::str_match::kmp_search;
use integer_encoding::{VarIntReader, VarIntWriter};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Triple {
    offset: u32,
    len: u32,
    char_value: char,
}

impl Triple {
    pub fn from_value(value: char) -> Self {
        Self {
            offset: 0,
            len: 0,
            char_value: value,
        }
    }

    pub fn no_traceback_return(&self) -> Option<char> {
        if self.offset == 0 && self.len == 0 {
            Some(self.char_value)
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! algc_encode {
    ($raw_string:expr, $window_size:expr, $process_func:expr) => {
        Codec::encode($raw_string, $window_size, ($process_func))
    };
}

#[macro_export]
macro_rules! algc_decode {
    ($encode_triples:expr, $process_func:expr) => {
        Codec::decode($encode_triples, ($process_func))
    };
}

impl From<Vec<u8>> for Triple {
    fn from(bytes: Vec<u8>) -> Self {
        let mut reader: &[u8] = bytes.as_ref();
        let offset = reader.read_varint::<u32>().expect("Found invalid u32");
        let len = reader.read_varint::<u32>().expect("Found invalid u32");
        let mut char_dest: Vec<u8> = Vec::with_capacity(3);
        reader
            .read_to_end(&mut char_dest)
            .expect("Found invalid character");

        let utf8_decode = String::from_utf8(char_dest).expect("Found invalid UTF-8");
        let char_value = utf8_decode.chars().last().unwrap();
        Self {
            offset,
            len,
            char_value,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<u8>> for Triple {
    fn into(self) -> Vec<u8> {
        let mut triple_bytes = Vec::with_capacity(128);
        triple_bytes
            .write_varint(self.offset)
            .expect("Encode Triple offset Error");
        triple_bytes
            .write_varint(self.len)
            .expect("Encode Triple len Error");

        let mut utf8_char = vec![0; self.char_value.len_utf8()];
        let _char_str = self.char_value.encode_utf8(&mut utf8_char);
        triple_bytes
            .write_all(&utf8_char)
            .expect("Triple char_value invalid UTF-8");
        triple_bytes
    }
}

#[derive(Debug, Clone)]
pub struct Codec {}

impl Codec {
    fn encode_triple<F, T>(remain: &[char], search: &[char], convert_func: F) -> (usize, T)
    where
        F: Fn(Triple) -> T,
    {
        let mut match_vec = Vec::new();
        let mut index = 0;
        let mut match_index = -1;
        for item in remain {
            match_vec.push(*item);
            let index_of = kmp_search(search, &match_vec);
            if index_of < 0 {
                break;
            } else {
                match_index = index_of;
            }
            index += 1;
        }
        let char_value = *match_vec.last().unwrap();
        let len = if index == 0 {
            0_u32
        } else {
            (match_vec.len() - 1) as u32
        };
        let start_index = search.len();
        let offset = if index > 0 {
            let curr_not_match = start_index + match_vec.len() - 1;
            let match_size = match_vec.len() - 1;
            let first_match_index = match_index as usize;
            (curr_not_match - match_size - first_match_index) as u32
        } else {
            0
        };
        let triple = Triple {
            offset,
            len,
            char_value,
        };
        (len as usize, convert_func(triple))
    }

    pub fn encode<F, T>(
        input: String,
        search_window_size: Option<usize>,
        encode_convert: F,
    ) -> Vec<T>
    where
        F: Fn(Triple) -> T,
    {
        let mut encode_triple_vec = Vec::new();
        let input_chars = input.chars().collect::<Vec<_>>();
        let input_len = input_chars.len();
        let mut index = 0;

        while index < input_len {
            let remain = &input_chars[index..input_len];
            let mut search_buf = &input_chars[0..index];
            if let Some(buffer_size) = search_window_size {
                if search_buf.len() > buffer_size {
                    search_buf = &input_chars[index - buffer_size..index];
                }
            }
            let triple_and_len = Codec::encode_triple(remain, search_buf, &encode_convert);
            let triple_len = triple_and_len.0;
            if triple_len == 0 {
                index += 1;
            } else {
                let curr = triple_len + index;
                let next = curr + 1;
                index = next;
            }
            encode_triple_vec.push(triple_and_len.1);
        }
        encode_triple_vec
    }

    pub fn decode<F, T>(encode_triple_vec: Vec<T>, convert_fn: F) -> String
    where
        F: Fn(T) -> Triple,
    {
        let mut result = String::new();
        for triple_value in encode_triple_vec {
            let triple = convert_fn(triple_value);
            if let Some(v) = triple.no_traceback_return() {
                result.push_str(v.to_string().as_str());
            } else {
                let start = result.chars().count() - triple.offset as usize;
                let end = start + triple.len as usize;
                let sub = &result.chars().take(end).skip(start).collect::<Vec<_>>();
                let mut append_str = result.clone();
                append_str.push_str(String::from_iter(sub).as_str());
                append_str.push_str(triple.char_value.to_string().as_str());
                result = append_str;
            }
        }
        result
    }
}

pub fn load_test_data() -> Vec<String> {
    let curr_dir = env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    let test_data_dir = curr_dir + "/src/test_data/input.data";
    let test_file_rs = File::open(test_data_dir.as_str());
    assert!(test_file_rs.is_ok());
    let test_file = test_file_rs.unwrap();

    let reader = BufReader::new(test_file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        assert!(line.is_ok());
        lines.push(line.unwrap());
    }
    lines
}

pub fn encode_decode_long_string(paragraph: String, window_size: usize) -> String {
    let encode_triple = algc_encode!(paragraph, Some(window_size), |triple: Triple| triple);
    algc_decode!(encode_triple, |triple| triple)
}

#[cfg(test)]
mod tests {
    use crate::codec::{encode_decode_long_string, load_test_data, Codec, Triple};
    use integer_encoding::{FixedInt, VarIntReader, VarIntWriter};
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::io::{Read, Write};

    #[test]
    fn test_encode_decode_bytes() {
        let input_str = "aaabbbccc|$%678";
        let encode_triple = algc_encode!(input_str.to_string(), None, |triple| triple.into());
        println!("encode_triple = {:?}", encode_triple);
        let decode_str = algc_decode!(encode_triple, |triple_binary: Vec<u8>| Triple::from(
            triple_binary
        ));
        println!(
            "input_string = {}, decode_string = {}",
            input_str, decode_str
        );
        assert_eq!(input_str, decode_str);
    }

    #[test]
    fn test_encode_decode() {
        let input_str = "aaa";
        let encode_triple = algc_encode!(input_str.to_string(), None, |triple| triple);
        println!("encode_triple={:?}", encode_triple);
        let decode_str = Codec::decode(encode_triple, |triple_value| triple_value);
        println!("decode_res = {}", decode_str);
        assert_eq!(decode_str, input_str);
    }

    #[test]
    fn test_emoji() {
        let emoji_str = "😓👌🏻👌🏻😭😭😁😁👌🏻";
        println!("input_emoji={:?}", emoji_str);
        let encode_triple = algc_encode!(emoji_str.to_string(), None, |triple| triple);
        let decode_emoji_str = Codec::decode(encode_triple, |triple_value| triple_value);
        println!("decode_emoji={:?}", decode_emoji_str);
        assert_eq!(emoji_str, decode_emoji_str);
    }

    fn codec_from_file_contents(file_contents: &[String]) -> Vec<String> {
        let mut decode_str_vec = Vec::new();
        for line in file_contents {
            let decode_string = encode_decode_long_string(line.clone(), 10);
            decode_str_vec.push(decode_string);
        }
        decode_str_vec
    }

    #[test]
    fn test_encode_decode_from_file() {
        let file_content = load_test_data();
        let decode_vec = codec_from_file_contents(&file_content);
        for (idx, decode_str) in decode_vec.iter().enumerate() {
            println!("decode_str = {}", decode_str);
            assert_eq!(file_content[idx], decode_str.clone());
        }
    }

    #[test]
    fn test_rand_string_encode_decode() {
        for _ in 0..1000 {
            let rand_str: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(13)
                .map(char::from)
                .collect();
            println!("random_str = {}", rand_str);
            let encode_triple = algc_encode!(rand_str.clone(), Some(4), |triple| triple);
            let decode_str = Codec::decode(encode_triple, |triple_value| triple_value);
            println!("decode_str = {}", decode_str.clone());
            assert_eq!(rand_str, decode_str);
        }
    }

    #[test]
    fn test_one_paragraph() {
        let paragraph = r##"We define a function in Rust by entering fn followed by a function
        name and a set of parentheses. The curly brackets tell the compiler where the function body
        begins and ends. We can call any function we’ve defined by entering its name followed by a
        set of parentheses. Because another_function is defined in the program, it can be called
        from inside the main function. Note that we defined another_function after the main
        function in the source code; we could have defined it before as well. Rust doesnt
        care where you define your functions, only that theyre defined somewhere."##;
        let codec_triple = algc_encode!(paragraph.to_string(), Some(10), |triple| triple);
        let decode_str = Codec::decode(codec_triple, |triple_vec| triple_vec);
        assert_eq!(paragraph.to_string(), decode_str);
    }

    #[test]
    fn test_multi_encode_decode() {
        let raw_input_vec = vec![
            "a",
            "aaabbb",
            "ababcbababaa",
            "ababcbababacbaaa",
            "0123456789999",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ];
        for input in raw_input_vec {
            let input_str = input.to_string();
            iter_str_idx(input_str.clone());
            let codec_triple = algc_encode!(input_str.clone(), Some(3), |triple| triple);
            println!("code_triple = {:#?}", codec_triple);
            let decode_str = Codec::decode(codec_triple, |triple_vec| triple_vec);
            println!("decode_str = {}", decode_str.clone());
            assert_eq!(input_str, decode_str.as_str());
        }
    }

    #[test]
    fn test_encode() {
        let expect_codec_triple = vec![
            Triple {
                offset: 0,
                len: 0,
                char_value: 'a',
            },
            Triple {
                offset: 0,
                len: 0,
                char_value: 'b',
            },
            Triple {
                offset: 2,
                len: 2,
                char_value: 'c',
            },
            Triple {
                offset: 4,
                len: 3,
                char_value: 'a',
            },
            Triple {
                offset: 8,
                len: 2,
                char_value: 'a',
            },
        ];
        let codec_triple = algc_encode!("ababcbababaa".to_string(), None, |triple| triple);
        for (pos, triple) in codec_triple.iter().enumerate() {
            assert_eq!(triple.clone(), expect_codec_triple[pos]);
        }
    }

    fn iter_str_idx(str: String) {
        let char_iter = str.chars();
        let pairs: Vec<(char, usize)> = char_iter
            .into_iter()
            .enumerate()
            .map(|pair| (pair.1, pair.0))
            .collect();
        println!("{:?}", pairs);
    }

    #[test]
    fn test_decode() {
        let encode_vec = vec![
            (0, 0, 'a'),
            (0, 0, 'b'),
            (2, 2, 'c'),
            (4, 3, 'a'),
            (8, 2, 'a'),
        ];
        let triple_values: Vec<Triple> = encode_vec
            .iter()
            .map(|tuple| Triple {
                offset: tuple.0,
                len: tuple.1,
                char_value: tuple.2,
            })
            .collect();
        let raw_str = Codec::decode(triple_values, |triple_vec| triple_vec);
        println!("raw_str = {:?}", raw_str);
        assert_eq!("ababcbababaa", raw_str);
    }

    #[test]
    fn test_triple_to_bytes() {
        let triple = Triple {
            offset: 0,
            len: 0,
            char_value: 'A',
        };

        let char_len = triple.char_value.len_utf8();
        let mut triple_bytes_buf = Vec::with_capacity(128);

        assert!(triple_bytes_buf.write_varint(triple.offset as u32).is_ok());
        assert!(triple_bytes_buf.write_varint(triple.len as u32).is_ok());
        let mut char_fixed_vec = vec![0; char_len];
        char_len.encode_fixed_vec();
        triple.char_value.encode_utf8(&mut char_fixed_vec);
        assert!(triple_bytes_buf.write_all(&char_fixed_vec).is_ok());

        let mut reader: &[u8] = triple_bytes_buf.as_ref();
        let offset = reader.read_varint::<u32>().unwrap();
        let len = reader.read_varint::<u32>().unwrap();

        let mut char_dest: Vec<u8> = vec![0; char_len];
        assert!(reader.read_to_end(&mut char_dest).is_ok());

        let utf8_decode = String::from_utf8(char_dest).expect("Found invalid UTF-8");

        let char_value = utf8_decode.chars().last().unwrap();
        println!("curr char = {:?}", char_value);

        let triple_de = Triple {
            offset,
            len,
            char_value,
        };
        assert_eq!(triple, triple_de);
    }
}
