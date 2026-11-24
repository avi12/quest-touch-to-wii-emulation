//! The UDP socket: answers requests and sends pad data to subscribed clients.

use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Instant;

mod clients;

use crate::wire::frame::{Magic, frame_dsu};
use crate::wire::pad::{PadState, pad_body};
use crate::wire::protocol::{
    CLIENT_TIMEOUT, Event, PROTOCOL_VERSION, RECENTER_HOLD, SERVER_ID_BASE, SLOT_INFO,
    parse_request,
};
use clients::ClientTable;

/// Large enough for any request a DSU client sends.
const RECV_BUFFER_BYTES: usize = 2048;

/// A DSU pad server on loopback, driven from the caller's loop (non-blocking).
#[derive(Debug)]
pub struct Server {
    socket: UdpSocket,
    source_id: u32,
    clients: ClientTable,
    packet_number: u32,
    /// While in the future, the PS button is held (= Dolphin's IMU "Recenter").
    pub recenter_until: Instant,
}

impl Server {
    /// Binds `127.0.0.1:port`; the PS button is held for the first second.
    pub fn bind(port: u16) -> io::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, port))?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            source_id: SERVER_ID_BASE.wrapping_add(u32::from(port)),
            clients: ClientTable::new(CLIENT_TIMEOUT),
            packet_number: 0,
            recenter_until: Instant::now() + RECENTER_HOLD,
        })
    }

    /// The bound address (useful when bound to port 0).
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// How many clients are subscribed to pad data.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Handles every request waiting on the socket, then returns.
    pub fn poll(&mut self) -> io::Result<()> {
        let mut buffer = [0; RECV_BUFFER_BYTES];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((len, from)) => {
                    let data = buffer.get(..len).unwrap_or_default();
                    self.handle_request(data, from)?;
                }
                // ICMP port-unreachable after a client quits surfaces here on Windows.
                Err(err) if err.kind() == ErrorKind::ConnectionReset => {}
                Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(err) => return Err(err),
            }
        }
    }

    /// Sends one pad snapshot to every live client, dropping those silent too long.
    pub fn send_pad(&mut self, pad: &PadState) -> io::Result<()> {
        self.packet_number = self.packet_number.wrapping_add(1);
        let now = Instant::now();
        let packet = pad_packet(
            self.source_id,
            &pad_body(pad, self.packet_number, now < self.recenter_until),
        )?;
        for client in self.clients.live(now) {
            // A vanished client is the timeout's job, like the TypeScript socket's error handler.
            let _ = self.socket.send_to(&packet, client);
        }
        Ok(())
    }

    fn handle_request(&mut self, data: &[u8], from: SocketAddr) -> io::Result<()> {
        let Some(event) = parse_request(data) else {
            return Ok(());
        };
        let body = match event {
            Event::Pad => {
                self.clients.touch(from, Instant::now());
                return Ok(());
            }
            Event::Version => PROTOCOL_VERSION.to_le_bytes().to_vec(),
            Event::Ports => [SLOT_INFO.as_slice(), &[0]].concat(),
        };
        let reply =
            frame_dsu(Magic::Server, self.source_id, event, &body).map_err(io::Error::other)?;
        self.socket.send_to(&reply, from).map(|_sent| ())
    }
}

fn pad_packet(source_id: u32, body: &[u8]) -> io::Result<Vec<u8>> {
    frame_dsu(Magic::Server, source_id, Event::Pad, body).map_err(io::Error::other)
}

#[cfg(test)]
mod tests;
