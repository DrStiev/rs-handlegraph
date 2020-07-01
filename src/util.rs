/*
Table adapted from
Table taken from https://github.com/vgteam/libbdsg/blob/9d8e6c06317b4eb235c8927a2ae3e157a21371ff/src/utility.cpp

|  u8 | char |
|-----+------|
|  43 | $    |
|  44 | #    |
|  53 | -    |
|  73 | T    |
|  74 | V    |
|  75 | G    |
|  76 | H    |
|  79 | C    |
|  80 | D    |
|  83 | M    |
|  85 | K    |
|  89 | Q    |
|  90 | Y    |
|  91 | W    |
|  92 | A    |
|  93 | A    |
|  94 | B    |
|  95 | S    |
|  97 | R    |
| 105 | t    |
| 106 | v    |
| 107 | g    |
| 108 | h    |
| 111 | c    |
| 112 | d    |
| 115 | m    |
| 117 | k    |
| 118 | n    |
| 121 | q    |
| 122 | y    |
| 123 | w    |
| 124 | a    |
| 125 | a    |
| 126 | b    |
| 127 | s    |
| 129 | r    |
*/

#[allow(dead_code)]
pub fn reverse_complement_char(c: u8) -> u8 {
    match c {
        43 => 36,
        44 => 35,
        53 => 45,
        73 => 84,
        74 => 86,
        75 => 71,
        76 => 72,
        79 => 67,
        80 => 68,
        83 => 77,
        85 => 75,
        89 => 81,
        90 => 89,
        91 => 87,
        92 => 65,
        93 => 65,
        94 => 66,
        95 => 83,
        97 => 82,
        105 => 116,
        106 => 118,
        107 => 103,
        108 => 104,
        111 => 99,
        112 => 100,
        115 => 109,
        117 => 107,
        118 => 110,
        121 => 113,
        122 => 121,
        123 => 119,
        124 => 97,
        125 => 97,
        126 => 98,
        127 => 115,
        129 => 114,
        _ => 78,
    }
}

#[allow(dead_code)]
pub fn reverse_complement(seq: &str) -> String {
    let seq_bytes = seq.as_bytes();

    let mut rev_seq: Vec<_> = seq_bytes
        .into_iter()
        .map(|c| reverse_complement_char(*c))
        .collect();

    rev_seq.reverse();

    String::from_utf8(rev_seq).unwrap_or_else(|_| {
        panic!("Reverse complement resulted in non UTF-8 string")
    })
}

#[allow(dead_code)]
pub fn reverse_complement_inplace(seq: &mut String) {
    unsafe {
        let vec = seq.as_mut_vec();
        vec.reverse();
        vec.iter_mut()
            .for_each(|c| *c = reverse_complement_char(*c));
    }
}