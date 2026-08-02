// copy-pasted from RFC
pub(crate) fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut chunks = data.chunks_exact(2);

    // Sum into u32 to keep carry
    for chunk in &mut chunks {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }

    // If number of bytes is odd, interpret it as high byte of u16 (BE)
    if let Some(&byte) = chunks.remainder().first() {
        sum += (byte as u32) << 8;
    }

    // Propagate carry in one's complement
    while sum > u16::MAX as u32 {
        sum = (sum & u16::MAX as u32) + (sum >> 16);
    }

    // Inverse
    !(sum as u16)
}
