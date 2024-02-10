
mod checksum_table;

pub fn generate_checksum(buffer: impl Into<Vec<u8>>, start: usize, stop: usize, sequence: u32) -> u16 {
    let buffer = buffer.into();
    let mut chkb: [u8; 64] = [0;64];
    let mut length = stop - start;
    if length > 60 {
        length = 60;
    }

    if let Some(chunk) = chkb.chunks_mut(length).next() {
        chunk.clone_from_slice(&buffer[start..start+length]);
    }
    let p: usize =   sequence as usize % (1024 -4);
    let check_table = &checksum_table::CHECKSUM_TABLE;
    chkb[length] = (sequence & 0xff) as u8  ^ check_table[p];
    chkb[length + 1] = check_table[p + 1];
    chkb[length + 2] = (sequence >> 8) as u8 ^ check_table[p + 2];
    chkb[length + 3] = check_table[p + 3];
    length += 4;

    block(chkb, length)
}

pub fn block(buffer: impl Into<Vec<u8>>, length: usize) -> u16 {
    let cst = &checksum_table::CHECKSUM_TABLE_U16;
    let mut crc: u16 = 0xffff;
    let buffer = buffer.into();
    for b in buffer.iter().take(length) {
        crc = (crc << 8) ^ cst[((crc >> 8) as u8 ^ b) as usize];
    }
    crc
}
