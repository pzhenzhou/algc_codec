use crate::str_match::kmp_search;

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

pub fn triple_offset(match_tables: &[MatchTable]) -> usize {
    if match_tables.len() == 1 {
        match_tables[0].match_range.1 - match_tables[0].first_match_index as usize
    } else {
        let match_table_len = match_tables.len();
        let last = &match_tables[match_table_len - 1];
        last.curr_not_match - last.match_size - last.first_match_index as usize
    }
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

    fn encode_triple(remain: String, search_pattern: String) -> (Option<MatchTable>, EncodeTriple) {
        let remain_vec = &remain.as_bytes().to_vec();
        let input_pattern = &search_pattern.as_bytes().to_vec();
        let mut match_sequence = Vec::new();
        let mut index = 0;
        let mut match_index = -1;
        for item in remain_vec {
            match_sequence.push(*item);
            let index_of = kmp_search(input_pattern, &match_sequence);
            if index_of < 0 {
                break;
            } else {
                match_index = index_of;
            }
            index += 1;
        }
        //for now emoji utf8 is not supported. will panic.
        let res = String::from_utf8_lossy(&match_sequence).to_string();
        let not_match_char = res.chars().last().unwrap();
        let len = if index == 0 { 0 } else { res.len() - 1 };
        let start_index = search_pattern.len();
        let match_table = if index > 0 {
            let match_table = MatchTable {
                match_sequence: res[0..res.len() - 1].to_string(),
                match_range: (start_index, start_index + index - 1),
                first_match_index: match_index,
                curr_not_match: start_index + match_sequence.len() - 1,
                next_index: start_index + match_sequence.len(),
                match_size: match_sequence.len() - 1,
            };
            Some(match_table)
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

    pub fn default_encode(&self) -> Vec<EncodeTriple> {
        let mut encode_triple_vec = Vec::new();
        let input_str = &self.input_string.as_bytes().to_vec();
        let input_len = input_str.len();
        let mut index = 0;
        let mut match_table_vec = Vec::new();
        while index < input_len {
            let remain = &self.input_string[index..input_len];
            let search_buf = &self.input_string[0..index];
            let triple_and_mtable =
                Codec::encode_triple(remain.to_string(), search_buf.to_string());
            let mut triple = triple_and_mtable.1;
            if let Some(m_table) = triple_and_mtable.0 {
                match_table_vec.push(m_table);
            }
            if triple.len == 0 {
                index += 1;
            } else {
                let curr = triple.len + search_buf.len();
                let next = curr + 1;
                if search_buf.len() == triple.len {
                    triple.offset = triple.len;
                } else {
                    triple.offset = triple_offset(&match_table_vec);
                    // println!("match_table = {:#?}", match_table_vec);
                    // println!("curr{},next={},triple={:?}", curr, next, triple);
                }
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
                let start = result.len() - triple.offset;
                let end = start + triple.len;
                let sub = &result[start..end];
                let mut append_str = result.clone();
                append_str.push_str(sub);
                append_str.push_str(triple.char_value.to_string().as_str());
                result = append_str;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::codec::{Codec, EncodeTriple};
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    const EXPECT_STR: &str = "ababcbababaa";

    #[test]
    fn test_encode_decode() {
        let input_str = "aaa";
        let codec = Codec::new(input_str.to_string());
        let encode_triple = codec.default_encode();
        println!("encode_triple={:?}", encode_triple);
        let decode_str = Codec::decode(&encode_triple);
        println!("decode_res = {}", decode_str);
        assert_eq!(decode_str, input_str.to_string());
    }

    #[test]
    #[should_panic]
    fn test_emoji_panic() {
        let emoji_str = "💖💖";
        let codec = Codec::new(emoji_str.to_string());
        codec.default_encode();
    }

    #[test]
    fn test_rand_string_encode_decode() {
        for _ in 0..5 {
            let rand_str: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(13)
                .map(char::from)
                .collect();
            println!("random_str = {:?}", rand_str);
            let codec = Codec::new(rand_str.clone());
            let encode_triple = codec.default_encode();
            let decode_str = Codec::decode(&encode_triple);
            println!("decode_str = {}", decode_str.clone());
            assert_eq!(rand_str, decode_str);
        }
    }

    #[test]
    fn test_multi_encode_decode() {
        let raw_input_vec = vec![
            "a",
            "aaabbb",
            "ababcbababaa",
            "0123456789999",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ];
        for input in raw_input_vec {
            let input_str = input.to_string();
            iter_str_idx(input_str.clone());
            let codec = Codec::new(input_str.clone());
            let codec_triple = codec.default_encode();
            println!("code_triple = {:?}", codec_triple);
            let decode_str = Codec::decode(&codec_triple);
            println!("decode_str = {}", decode_str.clone());
            assert_eq!(input_str, decode_str.as_str());
        }
    }

    #[test]
    fn test_encode() {
        let codec = Codec::new(EXPECT_STR.to_string());
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
        let codec_triple = codec.default_encode();
        println!("triple_vec = {:?}", codec_triple);
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
        assert_eq!(EXPECT_STR, raw_str);
    }
}
