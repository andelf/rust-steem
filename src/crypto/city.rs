extern "C" {
    // Hash function for a byte array.
    fn CCityHashCrc128(s: *const u8, len: u64, h: *mut u8);
}

pub fn city_hash_crc_128(s: &[u8]) -> [u8; 16] {
    let mut h = [0u8; 16];
    unsafe {
        CCityHashCrc128(s.as_ptr(), s.len() as u64, h.as_mut_ptr());
    };

    /*
    let first = LE::read_u64(&h[..8]);
    let second = LE::read_u64(&h[8..]);
    println!("first => {} second => {}", first, second);
    */

    h
}
