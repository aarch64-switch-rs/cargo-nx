//! The _nx-link stdio_ server implementation.
//!
//! If the NRO app uses the _libnx's nxlink stdio_ feature, it will redirect the stdout and stderr
//! streams over TCP.
//!
//!
//! This allows the NRO app to write to a remote console.

use tokio::{
    io,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, ToSocketAddrs},
};

/// Start the _nxlink stdio_ server.
///
/// This function listens for incoming TCP connections on the _nxlink_ client port and redirects
/// the data to the specified writer.
///
/// <div class="warning">
/// The libnx _nxlink_ runtime expects a TCP server listening at port `28771`.
///
/// See: https://github.com/switchbrew/libnx/blob/a063ceb19c3878d67eabd895ec7f76b3e93034e8/nx/source/runtime/nxlink_stdio.c#L41-L44
/// </div>
pub async fn start_server<A: ToSocketAddrs>(addr: A) -> io::Result<()> {
    let listener = TcpListener::bind(&addr).await?;
    let (stream, _) = listener.accept().await?;

    tracing::debug!("connection accepted from {}", stream.peer_addr()?);
    handle_stream(stream).await
}

/// Redirect the TCP stream to the Stdout stream.
async fn handle_stream<S>(mut stream: S) -> io::Result<()>
where
    S: AsyncRead + Unpin,
{
    let mut buffer = [0u8; 1024];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                tracing::debug!("connection closed");
                break;
            }
            Ok(len) => {
                io::stdout().write_all(&buffer[..len]).await?;
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}
