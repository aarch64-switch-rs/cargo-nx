//! Send an NRO file to the _netloader_ server.
//!
//! This module provides functions to send an NRO file to the _netloader_ server. The server will
//! save the file with the specified name if available space permits and will execute the file
//! afterward.

use std::{
    io,
    io::{BufReader, Cursor, Read, Write},
};

use flate2::{bufread::ZlibEncoder, Compression};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

/// The maximum file chunk size to compress and send to the server.
///
/// See: https://github.com/switchbrew/nx-hbmenu/blob/b7bcf3a9ece8f4717acabc8b9510e6a31a3efc1c/common/netloader.c#L35
const MAX_FILE_CHUNK_SIZE: usize = 0x4000;

/// The maximum NRO command-line arguments buffer size.
///
/// See: https://github.com/switchbrew/switch-tools/blob/22756068dd0ed6ff9734c59cb4f99ebd3f62555b/src/nxlink.c#L43
const MAX_CMD_BUF_SIZE: usize = 3072;

/// Send a file to the _netloader_ server.
///
/// This function sends a file to the _netloader_ server at the specified IP address. The server
/// will save the file with `file_name` if available space permits. The file is sent in chunks of
/// compressed data using the _deflate_ algorithm.
pub async fn send_nro_file<A: ToSocketAddrs, R: Read>(
    dst: A,
    file_name: &str,
    file_reader: &mut R,
    file_length: usize,
    cmd_args: impl AsRef<[String]>,
) -> io::Result<()> {
    let mut sock = TcpStream::connect(dst).await?;
    send_file_name_and_length(&mut sock, file_name, file_length).await?;
    compress_and_send_nro_file_data(&mut sock, file_reader, file_length).await?;
    send_nro_args(&mut sock, cmd_args).await?;
    Ok(())
}

/// Send the file name and length to the _netloader_ server.
///
/// This function sends the file name and size to the _netloader_ server. The server will respond
/// with an acknowledgement code.
async fn send_file_name_and_length<S>(
    stream: &mut S,
    file_name: &str,
    file_length: usize,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    // Send the file name (length-prefixed)
    write_length_prefixed(stream, file_name).await?;

    // Send the file length
    stream.write_u32_le(file_length as u32).await?;

    // Wait and check the acknowledgement code
    let rc = stream.read_i32_le().await?;
    match rc {
        0 => Ok(()),
        _ => Err(io::Error::new(io::ErrorKind::Other, SendNroError::from(rc))),
    }
}

/// Send the file content to the _nxlink_ server compressed with the deflate algorithm.
///
/// This function sends the file content to the _nxlink_ server compressed with the deflate
/// algorithm. The server will respond with an acknowledgement code.
async fn compress_and_send_nro_file_data<S, R>(
    stream: &mut S,
    file_reader: &mut R,
    file_length: usize,
) -> io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + ?Sized,
    R: Read,
{
    let mut encoder = ZlibEncoder::new(BufReader::new(file_reader), Compression::default());

    loop {
        // Read a data chunk from the file
        let mut buf = [0u8; MAX_FILE_CHUNK_SIZE];
        let read_len = encoder.read(&mut buf)?;
        if read_len == 0 {
            break;
        }

        // Send the compressed data chunk (length-prefixed)
        write_length_prefixed(stream, &buf[..read_len]).await?;

        // Log the progress
        let bytes_sent = encoder.total_in();
        tracing::debug!(
            "{} bytes sent ({:.2}%)",
            bytes_sent,
            (bytes_sent as f64 * 100.0) / file_length as f64,
        );
    }

    // Wait and check the response code
    let rc = stream.read_i32_le().await?;
    if rc != 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "Unknown error"));
    }

    Ok(())
}

/// Send the NRO command-line arguments to the _nxlink_ server
async fn send_nro_args<S>(stream: &mut S, args: impl AsRef<[String]>) -> io::Result<()>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    let mut cmd_buf = Cursor::new([0u8; MAX_CMD_BUF_SIZE]);

    // Write the command-line arguments to the buffer
    for arg in args.as_ref() {
        let arg_bytes = arg.as_bytes();

        // Check if the argument fits in the buffer, otherwise break
        if cmd_buf.position() as usize + arg_bytes.len() + 1 > MAX_CMD_BUF_SIZE {
            break;
        }

        // Write the argument to the buffer (null-terminated)
        cmd_buf.write_all(arg_bytes)?;
        cmd_buf.write_all(&[0u8])?;
    }

    // Get the command-line arguments buffer
    let cmd_buf_len = cmd_buf.position() as usize;
    let cmd_buf = cmd_buf.into_inner();

    // Send the command-line arguments (length-prefixed)
    write_length_prefixed(stream, &cmd_buf[..cmd_buf_len]).await?;

    Ok(())
}

/// Errors that can occur when sending a NRO file to the _netloader_ server.
#[derive(Debug, thiserror::Error)]
pub enum SendNroError {
    /// Failed to create file.
    ///
    /// An error returned by the _netloader_ server.
    #[error("Failed to create file")]
    CouldNotCreateFile,

    /// Insufficient space.
    ///
    /// An error returned by the _netloader_ server.
    #[error("Insufficient space")]
    InsufficientSpace,

    /// File-extension not recognized.
    ///
    /// Am error returned by the _netloader_ server.
    #[error("File-extension not recognized")]
    FileExtensionNotRecognized,

    /// Unknown error.
    ///
    /// An error returned by the _netloader_ server.
    #[error("Unknown error: {0}")]
    UnknownError(i32),
}

impl From<i32> for SendNroError {
    fn from(value: i32) -> Self {
        // NOTE: If built with `debug_assertions` enabled, this will panic if the value is 0.
        debug_assert!(value != 0, "unexpected success code");
        match value {
            -1 => Self::CouldNotCreateFile,
            -2 => Self::InsufficientSpace,
            -3 => Self::FileExtensionNotRecognized,
            _ => Self::UnknownError(value),
        }
    }
}

/// Write a length-prefixed data to the stream.
///
/// Writes the length of the data as a `u32` (little-endian) followed by the data bytes to the
/// stream.
async fn write_length_prefixed<S>(stream: &mut S, data: impl AsRef<[u8]>) -> io::Result<()>
where
    S: AsyncWrite + Unpin + ?Sized,
{
    let data = data.as_ref();
    let data_len = data.len() as u32;

    stream.write_u32_le(data_len).await?;
    stream.write_all(data).await?;

    Ok(())
}
