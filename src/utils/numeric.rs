const HEX_TABLE: [u8; 256] = {
    let mut table = [255; 256];

    let mut i = 0;
    while i < 10 {
        table[(b'0' + i) as usize] = i;
        i += 1;
    }

    let mut i = 0;
    while i < 6 {
        table[(b'a' + i) as usize] = 10 + i;
        table[(b'A' + i) as usize] = 10 + i;
        i += 1;
    }

    table
};

pub fn parse_hex_to_u64(s: &[u8]) -> Option<u64> {
    let hex_part = if s.len() >= 2 && (s[0] == b'0' && (s[1] == b'x' || s[1] == b'X')) {
        &s[2..]
    } else {
        s
    };

    if hex_part.is_empty() {
        return None;
    }

    let mut result: u64 = 0;
    for &byte in hex_part {
        let val = HEX_TABLE[byte as usize];

        if val == 255 {
            return None;
        }

        result = (result << 4) | (val as u64);
    }

    Some(result)
}

pub fn val_to_bit_mask(n: u64) -> u64 {
    if n == 0 {
        0
    } else {
        (1 << (64 - n.leading_zeros())) - 1
    }
}
