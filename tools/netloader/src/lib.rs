pub mod loader;
pub mod stdio;

/// The _netloader_ server port.
///
/// The _netloader_ server listens on this port for:
/// - _TCP_: Incoming file transfers.
/// - _UDP_: Discovery messages.
pub const SERVER_PORT: u16 = 28280;

/// The _netloader_ client port.
///
/// The server sends the response to the discovery message to this UDP port.
pub const CLIENT_PORT: u16 = 28771;
