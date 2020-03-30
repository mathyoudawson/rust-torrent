use sha1::{Sha1, Digest};

pub fn compute_sha1_hash(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.input(input);

    hasher.result().to_vec()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn computes_correct_hash()  {
        assert_eq!(compute_sha1_hash(vec![8u8]),
        vec![141, 136, 63, 21, 119, 202, 140, 51, 75, 124, 109, 117, 204, 183, 18, 9, 215, 28, 237, 19]);

    }

    #[test]
    fn computes_20_byte_char() {
        assert_eq!(compute_sha1_hash(vec![8u8]).len(),
        20
        )
    }
}

