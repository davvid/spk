// Copyright (c) 2021 Sony Pictures Imageworks, et al.
// SPDX-License-Identifier: Apache-2.0
// https://github.com/imageworks/spk

use std::{
    io::{BufRead, Read, Write},
    iter::FromIterator,
};

use super::hash::{Digest, DIGEST_SIZE, NULL_DIGEST};
use crate::{Error, Result};

pub const INT_SIZE: usize = std::mem::size_of::<u64>();

#[cfg(test)]
#[path = "./binary_test.rs"]
mod binary_test;

/// Read and validate the given header from a binary stream.
pub fn consume_header(mut reader: impl Read, header: &[u8]) -> Result<()> {
    let mut buf = vec![0; header.len() + 1];
    reader.read_exact(buf.as_mut_slice())?;
    if buf[0..header.len()] != *header || buf.last() != Some(&b'\n') {
        Err(Error::from(format!(
            "Invalid header: expected {:?}, got {:?}",
            header, buf
        )))
    } else {
        Ok(())
    }
}

/// Write an identifiable header to the given binary stream.
pub fn write_header(mut writer: impl Write, header: &[u8]) -> Result<()> {
    writer.write_all(header)?;
    writer.write_all(b"\n")?;
    Ok(())
}

/// Write an integer to the given binary stream.
pub fn write_int(mut writer: impl Write, value: i64) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read an integer from the given binary stream.
pub fn read_int(mut reader: impl Read) -> Result<i64> {
    let mut buf: [u8; INT_SIZE] = [0, 0, 0, 0, 0, 0, 0, 0];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

/// Write an unsigned integer to the given binary stream.
pub fn write_uint(mut writer: impl Write, value: u64) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Read an unsigned integer from the given binary stream.
pub fn read_uint(mut reader: impl Read) -> Result<u64> {
    let mut buf: [u8; INT_SIZE] = [0, 0, 0, 0, 0, 0, 0, 0];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

/// Write a digest to the given binary stream.
pub fn write_digest(mut writer: impl Write, digest: &Digest) -> Result<()> {
    writer.write_all(digest.as_ref())?;
    Ok(())
}

/// Read a digest from the given binary stream.
pub fn read_digest(mut reader: impl Read) -> Result<Digest> {
    let mut buf: [u8; DIGEST_SIZE] = NULL_DIGEST;
    reader.read_exact(buf.as_mut())?;
    Digest::from_bytes(&buf)
}

/// Write a string to the given binary stream.
pub fn write_string(mut writer: impl Write, string: &str) -> Result<()> {
    if string.contains('\x00') {
        return Err(Error::from(
            "Cannot encode string with null character".to_string(),
        ));
    }
    writer.write_all(string.as_bytes())?;
    writer.write_all("\x00".as_bytes())?;
    Ok(())
}

/// Read a string from the given binary stream.
pub fn read_string(reader: &mut impl BufRead) -> Result<String> {
    let mut r = Vec::with_capacity(
        // most strings are short enough that they are expected
        // to be fully read in one iteration, but we can get
        // unlucky with the string spanning two buffered reads.
        2,
    );
    loop {
        let buf = reader.fill_buf()?;
        match buf.iter().position(|&c| c == 0) {
            Some(index) => {
                r.push(std::str::from_utf8(&buf[..index])?.to_string());
                reader.consume(index + 1);
                break;
            }
            None => {
                if buf.is_empty() {
                    return Err(Error::from(std::io::Error::from(
                        std::io::ErrorKind::UnexpectedEof,
                    )));
                }
                r.push(std::str::from_utf8(buf)?.to_string());
                let l = buf.len();
                reader.consume(l)
            }
        }
    }
    Ok(String::from_iter(r.into_iter()))
}
