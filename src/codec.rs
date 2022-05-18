use crate::str_match::kmp_search;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Some fields are redundant and are more for testing purposes.
/// Record the position of each match and the position of the next start.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MatchTable {
    match_sequence: String,
    match_range: (usize, usize),
    first_match_index: i32,
    curr_not_match: usize,
    next_index: usize,
    match_size: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EncodeTriple {
    offset: usize,
    len: usize,
    char_value: char,
}

impl EncodeTriple {
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

#[derive(Debug, Clone)]
pub struct Codec {
    input_string: String,
}

impl Codec {
    pub fn new(input_string: String) -> Self {
        Self { input_string }
    }

    fn encode_triple(
        remain: &[char],
        search_pattern: &[char],
    ) -> (Option<MatchTable>, EncodeTriple) {
        let mut match_vec = Vec::new();
        let mut index = 0;
        let mut match_index = -1;
        for item in remain {
            match_vec.push(*item);
            let index_of = kmp_search(search_pattern, &match_vec);
            if index_of < 0 {
                break;
            } else {
                match_index = index_of;
            }
            index += 1;
        }
        let not_match_char = *match_vec.last().unwrap();
        let match_sequence = match_vec[0..match_vec.len()]
            .iter()
            .cloned()
            .collect::<String>();
        let len = if index == 0 { 0 } else { match_vec.len() - 1 };
        let start_index = search_pattern.len();
        let match_table = if index > 0 {
            Some(MatchTable {
                match_sequence,
                match_range: (start_index, start_index + index - 1),
                first_match_index: match_index,
                curr_not_match: start_index + match_vec.len() - 1,
                next_index: start_index + match_vec.len(),
                match_size: match_vec.len() - 1,
            })
        } else {
            None
        };
        (
            match_table,
            EncodeTriple {
                offset: 0,
                len,
                char_value: not_match_char,
            },
        )
    }

    pub fn default_encode(&self, search_window_size: Option<usize>) -> Vec<EncodeTriple> {
        let mut encode_triple_vec = Vec::new();
        let input_chars = &self.input_string.chars().collect::<Vec<_>>();
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
            let triple_and_match_tables = Codec::encode_triple(remain, search_buf);
            let mut triple = triple_and_match_tables.1;
            let match_table_opt = triple_and_match_tables.0;

            if triple.len == 0 {
                index += 1;
            } else {
                if let Some(last_match_table) = match_table_opt {
                    triple.offset = last_match_table.curr_not_match
                        - last_match_table.match_size
                        - last_match_table.first_match_index as usize;
                }
                let curr = triple.len + index;
                let next = curr + 1;
                index = next;
            }
            encode_triple_vec.push(triple);
        }
        encode_triple_vec
    }

    pub fn decode(encode_triple: &[EncodeTriple]) -> String {
        let mut result = String::new();
        for triple in encode_triple {
            if let Some(v) = triple.no_traceback_return() {
                result.push_str(v.to_string().as_str());
            } else {
                let start = result.chars().count() - triple.offset;
                let end = start + triple.len;
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
    let codec = Codec::new(paragraph);
    let encode_triple = codec.default_encode(Some(window_size));
    Codec::decode(&encode_triple)
}

#[cfg(test)]
mod tests {
    use crate::codec::{encode_decode_long_string, load_test_data, Codec, EncodeTriple};
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    #[test]
    fn test_encode_decode() {
        let input_str = "aaa";
        let codec = Codec::new(input_str.to_string());
        let encode_triple = codec.default_encode(None);
        println!("encode_triple={:?}", encode_triple);
        let decode_str = Codec::decode(&encode_triple);
        println!("decode_res = {}", decode_str);
        assert_eq!(decode_str, input_str);
    }

    #[test]
    fn test_emoji() {
        let emoji_str = "😓👌🏻👌🏻😭😭😁😁👌🏻";
        println!("input_emoji={:?}", emoji_str);
        let codec = Codec::new(emoji_str.to_string());
        let encode_triple = codec.default_encode(None);
        let decode_emoji_str = Codec::decode(&encode_triple);
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
            let codec = Codec::new(rand_str.clone());
            let encode_triple = codec.default_encode(Some(4));
            let decode_str = Codec::decode(&encode_triple);
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
        let codec = Codec::new(paragraph.to_string());
        let codec_triple = codec.default_encode(Some(4));
        let decode_str = Codec::decode(&codec_triple);
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
            let codec = Codec::new(input_str.clone());
            let codec_triple = codec.default_encode(Some(3));
            println!("code_triple = {:#?}", codec_triple);
            let decode_str = Codec::decode(&codec_triple);
            println!("decode_str = {}", decode_str.clone());
            assert_eq!(input_str, decode_str.as_str());
        }
    }

    #[test]
    fn test_encode() {
        let codec = Codec::new("ababcbababaa".to_string());
        let expect_codec_triple = vec![
            EncodeTriple {
                offset: 0,
                len: 0,
                char_value: 'a',
            },
            EncodeTriple {
                offset: 0,
                len: 0,
                char_value: 'b',
            },
            EncodeTriple {
                offset: 2,
                len: 2,
                char_value: 'c',
            },
            EncodeTriple {
                offset: 4,
                len: 3,
                char_value: 'a',
            },
            EncodeTriple {
                offset: 8,
                len: 2,
                char_value: 'a',
            },
        ];
        let codec_triple = codec.default_encode(None);
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
        let encode_triple_vec: Vec<EncodeTriple> = encode_vec
            .iter()
            .map(|tuple| EncodeTriple {
                offset: tuple.0,
                len: tuple.1,
                char_value: tuple.2,
            })
            .collect();
        let raw_str = Codec::decode(&encode_triple_vec);
        println!("raw_str = {:?}", raw_str);
        assert_eq!("ababcbababaa", raw_str);
    }
}
