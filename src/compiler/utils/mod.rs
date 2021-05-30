use seahash::hash;

pub fn hash_str(s: &str) -> u64 {
    hash(s.as_bytes())
}