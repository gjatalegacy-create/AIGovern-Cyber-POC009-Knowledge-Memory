fn crc32_parts(parts: &[&[u8]]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in parts.iter().flat_map(|part| part.iter()) {
        crc ^= b as u32;
        let mut i = 0;
        while i < 8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            i += 1;
        }
    }
    !crc
}
pub(crate) fn frame_crc(length: u32, payload: &[u8]) -> u32 {
    crc32_parts(&[&length.to_le_bytes(), payload])
}
