
const MAX_REPRS: [u64; 3] = [u8::MAX as u64, u16::MAX as u64, u32::MAX as u64];

pub fn write_count_bytes(count: u64, buffer: &mut Vec<u8>) {
    let mode = get_mode(count);

    buffer.push(mode as u8);

    match mode {
        0 => buffer.push(count as u8),
        1 => buffer.extend_from_slice(&u16::to_ne_bytes(count as u16)),
        2 => buffer.extend_from_slice(&u32::to_ne_bytes(count as u32)),
        _ => buffer.extend_from_slice(&u64::to_ne_bytes(count)),
    };
}

pub fn write_count_bytes_with_mode(count: u64, mode: usize, buffer: &mut Vec<u8>) {
    match mode {
        0 => buffer.push(count as u8),
        1 => buffer.extend_from_slice(&u16::to_ne_bytes(count as u16)),
        2 => buffer.extend_from_slice(&u32::to_ne_bytes(count as u32)),
        _ => buffer.extend_from_slice(&u64::to_ne_bytes(count)),
    };
}

pub fn get_mode(count: u64) -> usize {
    let mut mode = 0usize;

    while mode < 3 && count >= MAX_REPRS[mode] {
        mode += 1;
    }
    mode
}

pub fn read_count_bytes(buffer: &[u8]) -> (u64, usize) {
    let mode = buffer[0] as usize;
    let count = match mode {
        0 => buffer[1] as u64,
        1 => u16::from_ne_bytes([buffer[1], buffer[2]]) as u64,
        2 => u32::from_ne_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as u64,
        _ => u64::from_ne_bytes([buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7], buffer[8]]),
    };

    let bytes_read = match mode {
        0 => 2,
        1 => 3,
        2 => 5,
        _ => 9,
    };

    (count, bytes_read)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_count_u8() {
        let mut buffer = Vec::new();
        write_count_bytes(100, &mut buffer);
        let (count, bytes_read) = read_count_bytes(&buffer);
        assert_eq!(count, 100);
        assert_eq!(bytes_read, 2);
    }

    #[test]
    fn test_write_and_read_count_u16() {
        let mut buffer = Vec::new();
        write_count_bytes(1000, &mut buffer);
        let (count, bytes_read) = read_count_bytes(&buffer);
        assert_eq!(count, 1000);
        assert_eq!(bytes_read, 3);
    }

    #[test]
    fn test_write_and_read_count_u32() {
        let mut buffer = Vec::new();
        write_count_bytes(100000, &mut buffer);
        let (count, bytes_read) = read_count_bytes(&buffer);
        assert_eq!(count, 100000);
        assert_eq!(bytes_read, 5);
    }

    #[test]
    fn test_write_and_read_count_u64() {
        let mut buffer = Vec::new();
        write_count_bytes(10000000000, &mut buffer);
        let (count, bytes_read) = read_count_bytes(&buffer);
        assert_eq!(count, 10000000000);
        assert_eq!(bytes_read, 9);
    }

    #[test]
    fn test_write_and_read_count_boundary_u8_max() {
        let mut buffer = Vec::new();
        write_count_bytes(u8::MAX as u64, &mut buffer);
        let (count, _) = read_count_bytes(&buffer);
        assert_eq!(count, u8::MAX as u64);
    }

    #[test]
    fn test_write_and_read_count_boundary_u16_max() {
        let mut buffer = Vec::new();
        write_count_bytes(u16::MAX as u64, &mut buffer);
        let (count, _) = read_count_bytes(&buffer);
        assert_eq!(count, u16::MAX as u64);
    }
}
