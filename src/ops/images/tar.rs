//! Minimal POSIX ustar writer for one-file archives. Used to wrap a
//! Dockerfile as a /build context body without pulling in a tar crate.
//!
//! Single regular file, NUL-padded to 512 blocks, terminated with two
//! 512-byte zero blocks (per POSIX). Mode 0644, uid/gid 0, mtime now.
//! The header fields beyond what we set are zero-initialized — that's a
//! valid ustar shape for tools that read the archive.

use std::time::{SystemTime, UNIX_EPOCH};

const BLOCK: usize = 512;

pub(super) fn one_file(name: &[u8], content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(BLOCK + content.len() + 2 * BLOCK);
    out.extend_from_slice(&header(name, content.len() as u64, current_mtime()));
    out.extend_from_slice(content);
    let pad = (BLOCK - (content.len() % BLOCK)) % BLOCK;
    out.extend(std::iter::repeat_n(0u8, pad));
    // Two zero blocks: end-of-archive marker.
    out.extend(std::iter::repeat_n(0u8, 2 * BLOCK));
    out
}

fn header(name: &[u8], size: u64, mtime: u64) -> [u8; BLOCK] {
    let mut h = [0u8; BLOCK];
    let n = name.len().min(100);
    h[..n].copy_from_slice(&name[..n]);
    write_octal(&mut h[100..108], 0o644, 7);
    write_octal(&mut h[108..116], 0, 7);
    write_octal(&mut h[116..124], 0, 7);
    write_octal(&mut h[124..136], size, 11);
    write_octal(&mut h[136..148], mtime, 11);
    // Checksum field is 8 spaces during the sum, then "NNNNNN\0 ".
    h[148..156].fill(b' ');
    h[156] = b'0'; // typeflag = regular file
    h[257..263].copy_from_slice(b"ustar\0");
    h[263..265].copy_from_slice(b"00");
    let sum: u32 = h.iter().map(|&b| b as u32).sum();
    write_octal(&mut h[148..154], u64::from(sum), 6);
    h[154] = 0;
    h[155] = b' ';
    h
}

/// Write `value` as a zero-padded octal string into `slot[..len]`, then a
/// NUL terminator at `slot[len]` if there's room.
fn write_octal(slot: &mut [u8], value: u64, len: usize) {
    let s = format!("{value:0>width$o}", width = len);
    slot[..len].copy_from_slice(s.as_bytes());
    if slot.len() > len {
        slot[len] = 0;
    }
}

fn current_mtime() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_layout() {
        let h = header(b"Dockerfile", 42, 1_704_067_200);
        assert_eq!(&h[..10], b"Dockerfile");
        assert_eq!(&h[100..107], b"0000644");
        assert_eq!(h[107], 0);
        assert_eq!(&h[124..135], b"00000000052"); // 42 octal
        assert_eq!(&h[136..147], b"14544400200");
        assert_eq!(h[156], b'0');
        assert_eq!(&h[257..263], b"ustar\0");
        assert_eq!(&h[263..265], b"00");
        // Checksum re-validates: if we replace the 6 chksum digits with
        // spaces and re-sum, we should get the value the digits encoded.
        let stored: u32 = std::str::from_utf8(&h[148..154])
            .unwrap()
            .trim_end_matches('\0')
            .chars()
            .fold(0u32, |acc, c| acc * 8 + c.to_digit(8).unwrap());
        let mut probe = h;
        probe[148..156].fill(b' ');
        let computed: u32 = probe.iter().map(|&b| b as u32).sum();
        assert_eq!(stored, computed);
    }

    #[test]
    fn archive_shape() {
        let buf = one_file(b"Dockerfile", b"FROM alpine:3.19\n");
        // 1 header + 1 data block (content < 512) + 2 zero blocks
        assert_eq!(buf.len(), BLOCK * 4);
        assert_eq!(&buf[BLOCK..BLOCK + 17], b"FROM alpine:3.19\n");
        // Final 1024 bytes are all zero (end-of-archive).
        assert!(buf[BLOCK * 2..].iter().all(|&b| b == 0));
    }
}
