use std::net::{Ipv4Addr, UdpSocket};
use std::time::{Duration, Instant};

use super::{Server, pad_packet};
use crate::frame::{Magic, frame_dsu, hex};
use crate::pad::{pad_body, sample_pad};
use crate::protocol::{Event, SERVER_ID_BASE};

/// Golden datagrams captured from `Server.sendPad` in `src/dsu_server.ts` on port 26799.
const PAD_RECENTERING: &str = "44535553e9035400607c062c8d291270020010000002020100000000000105010100000011a8010080ff0040000000ff00ff00ffff000a14000000000000000000000000cb04fb711f010000cdcccc3d000080bf0000803e00004841333353c00000807f";
const PAD_RELEASED: &str = "44535553e903540076345b358d291270020010000002020100000000000105010200000011a8000080ff0040000000ff00ff00ffff000a14000000000000000000000000cb04fb711f010000cdcccc3d000080bf0000803e00004841333353c00000807f";
/// Golden replies from the same server to version and ports requests.
const VERSION_REPLY: &str = "44535553e9030600964c2ee18d29127000001000e903";
const PORTS_REPLY: &str = "44535553e9031000e116df1e8d29127001001000000202010000000000010500";
const GOLDEN_ID: u32 = SERVER_ID_BASE + 26799;

#[test]
fn pad_packets_match_typescript() {
    let pad = sample_pad();
    let recentering = pad_packet(GOLDEN_ID, &pad_body(&pad, 1, true)).unwrap();
    let released = pad_packet(GOLDEN_ID, &pad_body(&pad, 2, false)).unwrap();
    assert_eq!(hex(&recentering), PAD_RECENTERING);
    assert_eq!(hex(&released), PAD_RELEASED);
}

fn client_for(server: &Server) -> UdpSocket {
    let client = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    client.connect(server.local_addr().unwrap()).unwrap();
    client
}

fn request(client: &UdpSocket, event: Event) {
    let packet = frame_dsu(Magic::Client, 42, event, &[0; 4]).unwrap();
    client.send(&packet).unwrap();
}

/// Polls the server until the client receives a datagram.
fn receive(server: &mut Server, client: &UdpSocket) -> Vec<u8> {
    let mut buffer = [0; 256];
    for _attempt in 0..100 {
        server.poll().unwrap();
        if let Ok(len) = client.recv(&mut buffer) {
            return buffer[..len].to_vec();
        }
    }
    panic!("no datagram arrived");
}

/// Rewrites a golden datagram for another server id, so it can be compared with a live reply.
fn with_id(golden: &str, server_id: u32) -> Vec<u8> {
    let bytes: Vec<u8> = (0..golden.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&golden[i..i + 2], 16).unwrap())
        .collect();
    let event = Event::from_code(u32::from_le_bytes(bytes[16..20].try_into().unwrap())).unwrap();
    frame_dsu(Magic::Server, server_id, event, &bytes[20..]).unwrap()
}

/// Polls until the server has subscribed `count` clients or two seconds pass.
fn poll_for_clients(server: &mut Server, count: usize) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while server.client_count() != count && Instant::now() < deadline {
        server.poll().unwrap();
    }
}

#[test]
fn answers_version_and_ports_then_streams_pad() {
    let mut server = Server::bind(0).unwrap();
    let id = SERVER_ID_BASE; // bound to port 0
    let client = client_for(&server);

    request(&client, Event::Version);
    assert_eq!(receive(&mut server, &client), with_id(VERSION_REPLY, id));

    request(&client, Event::Ports);
    assert_eq!(receive(&mut server, &client), with_id(PORTS_REPLY, id));

    request(&client, Event::Pad);
    poll_for_clients(&mut server, 1);
    server.recenter_until = Instant::now();
    server.send_pad(&sample_pad()).unwrap();
    let expected = pad_packet(id, &pad_body(&sample_pad(), 1, false)).unwrap();
    assert_eq!(receive(&mut server, &client), expected);
}

#[test]
fn ignores_garbage() {
    let mut server = Server::bind(0).unwrap();
    let client = client_for(&server);
    client.send(b"DSUC short").unwrap();
    client
        .send(&frame_dsu(Magic::Server, 1, Event::Pad, &[]).unwrap())
        .unwrap();
    request(&client, Event::Version); // a valid request after the garbage proves it was read
    receive(&mut server, &client);
    assert_eq!(server.client_count(), 0);
}
