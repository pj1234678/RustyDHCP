

pub mod options;
pub mod packet;
pub mod server;

/// Converts a u32 to 4 bytes (Big endian)
#[macro_export]
macro_rules! u32_bytes {
    ( $x:expr ) => {
        [
            ($x >> 24) as u8,
            ($x >> 16) as u8,
            ($x >> 8) as u8,
            $x as u8,
        ]
    };
}

/// Converts 4 bytes to a u32 (Big endian)
#[macro_export]
macro_rules! bytes_u32 {
    ( $x:expr ) => {
        ($x[0] as u32) * (1 << 24)
            + ($x[1] as u32) * (1 << 16)
            + ($x[2] as u32) * (1 << 8)
            + ($x[3] as u32)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
