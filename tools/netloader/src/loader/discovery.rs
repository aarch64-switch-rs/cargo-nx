//! Implementation of the _netloader_ server discovery protocol.
//!
//! The _netloader_ server discovery protocol is used to discover the _netloader_ server in the
//! network using UDP broadcast messages.
//!
//! The client sends a broadcast message to the network to discover the server. The server responds
//! to the broadcast message with the same message. The client listens for the response and
//! determines the IP address of the server.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use tokio::{
    io,
    net::{ToSocketAddrs, UdpSocket},
};

use crate::{CLIENT_PORT, SERVER_PORT};

/// The discovery message to send.
///
/// If the _netloader_ was compiled with `PING_ENABLED`, the server will be listening on UDP port
/// `28280` for this message.
///
/// See: https://github.com/switchbrew/nx-hbmenu/blob/b7bcf3a9ece8f4717acabc8b9510e6a31a3efc1c/common/netloader.c#L633-L646
const PING_MESSAGE: &[u8] = b"nxboot";

/// The discovery message response to receive.
///
/// The _netloader_ server responds to the discovery message with this message.
///
/// See: https://github.com/switchbrew/nx-hbmenu/blob/b7bcf3a9ece8f4717acabc8b9510e6a31a3efc1c/common/netloader.c#L643
const PONG_MESSAGE: &[u8] = b"bootnx";

/// The broadcast address to send the discovery message.
///
/// The _netloader_ server listens on UDP port `28280` for the discovery message.
const BROADCAST_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::BROADCAST, SERVER_PORT);

/// The address to bind for receiving the discovery response.
///
/// The _netloader_ server responds to the discovery message on UDP port `28771`.
///
/// See: https://github.com/switchbrew/nx-hbmenu/blob/b7bcf3a9ece8f4717acabc8b9510e6a31a3efc1c/common/netloader.c#L534-539
const RECEIVE_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, CLIENT_PORT);

/// Discover the _neloader_ server in the network.
///
/// This function sends a broadcast message over UDP to discover the _netloader_ server.
/// It waits for a response within a specified timeout period and returns the IP address
/// of the discovered server if found.
///
/// # Returns
///
///  * `Ok(Some(IpAddr))` - The IP address of the discovered server.
///  * `Ok(None)` - No server was discovered.
///  * `Err(io::Error)` - An error occurred during the discovery process.
///
/// # Errors
///
/// This function will return an error if:
///  * The UDP socket cannot be bound to an address.
///  * The socket cannot be set to broadcast mode.
///  * The discovery message cannot be sent.
///  * There is an error receiving the response.
pub async fn discover(timeout: Duration, retries: u32) -> io::Result<Option<IpAddr>> {
    // Create UDP socket for broadcasting the discovery message. Set it to broadcast mode.
    let broadcast_socket = UdpSocket::bind("0.0.0.0:0").await?;
    broadcast_socket.set_broadcast(true)?;

    // Create UDP socket for receiving the response at `0.0.0.0:28771`
    let receive_socket = UdpSocket::bind(RECEIVE_ADDR).await?;

    for attempt in 0..retries {
        let ping_fut = async {
            // Send a broadcast message to discover the server in the network
            tracing::debug!(%attempt, "sending ping message");
            if let Err(error) = send_ping_message(&broadcast_socket, BROADCAST_ADDR).await {
                tracing::debug!(%attempt, ?error, "sendto error");
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    DiscoveryError::SendPingFailed(error),
                ));
            }

            // Wait for a response from the server
            tracing::debug!(%attempt, "waiting pong response");
            match recv_pong_response(&receive_socket).await {
                Ok(res) => Ok(res),
                Err(error) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    DiscoveryError::RecvPongFailed(error),
                )),
            }
        };

        // Run the ping future with a timeout
        match tokio::time::timeout(timeout, ping_fut).await {
            Ok(res) => match res {
                Ok(ip_addr) => {
                    return Ok(Some(ip_addr));
                }
                // If we reached the max number of retries, return an error
                Err(err) if attempt + 1 == retries => {
                    return Err(err);
                }
                Err(_) => continue,
            },
            // If the timeout was reached, retry
            Err(_) => continue,
        }
    }

    Ok(None)
}

/// Send the discovery ping message to the target address.
async fn send_ping_message<A: ToSocketAddrs>(socket: &UdpSocket, target: A) -> io::Result<()> {
    socket.send_to(PING_MESSAGE, target).await?;
    Ok(())
}

/// Receive the discovery pong message (ping response) from the server.
///
/// Returns the IP address of the sender if the message is valid. Otherwise, returns `None`.
async fn recv_pong_response(socket: &UdpSocket) -> io::Result<IpAddr> {
    let mut buf = [0u8; 0x10];
    let (len, addr) = socket.recv_from(&mut buf).await?;

    if len >= PING_MESSAGE.len() && &buf[0..PONG_MESSAGE.len()] == PONG_MESSAGE {
        Ok(addr.ip())
    } else {
        tracing::debug!(
            "invalid response message: '{}'",
            String::from_utf8_lossy(&buf[..len])
        );
        Err(io::Error::new(
            io::ErrorKind::Other,
            DiscoveryError::InvalidResponse,
        ))
    }
}

/// An error that occurred during the discovery process.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// An error occurred while binding the UDP socket.
    #[error(transparent)]
    BindFailed(io::Error),
    /// An error occurred while sending the discovery message.
    #[error(transparent)]
    SendPingFailed(io::Error),
    /// An error occurred while receiving the discovery response.
    #[error(transparent)]
    RecvPongFailed(io::Error),
    /// The received message was invalid.
    #[error("invalid response message")]
    InvalidResponse,
    /// The max number of retries was reached without discovering the server.
    #[error("discovery retries exhausted")]
    RetriesExhausted,
}
