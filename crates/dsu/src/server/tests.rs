use std::net::{Ipv4Addr, UdpSocket};
use std::time::{Duration, Instant};

use crate::server::{Server, pad_packet};
use crate::wire::frame::{EVENT_RANGE, HEADER_BYTES, Magic, SourceId, frame_dsu, hex};
use crate::wire::pad::{SAMPLE_PAD, pad_body};
use crate::wire::protocol::{Event, SERVER_ID_BASE};

/// Golden datagrams captured from `Server.sendPad` in `src/dsu_server.ts` on port 26799.
const PAD_RECENTERING: &str = "44535553e9035400607c062c8d291270020010000002020100000000000105010100000011a8010080ff0040000000ff00ff00ffff000a14000000000000000000000000cb04fb711f010000cdcccc3d000080bf0000803e00004841333353c00000807f";
const PAD_RELEASED: &str = "44535553e903540076345b358d291270020010000002020100000000000105010200000011a8000080ff0040000000ff00ff00ffff000a14000000000000000000000000cb04fb711f010000cdcccc3d000080bf0000803e00004841333353c00000807f";
/// Golden replies from the same server to version and ports requests.
const VERSION_REPLY: &str = "44535553e9030600964c2ee18d29127000001000e903";
const PORTS_REPLY: &str = "44535553e9031000e116df1e8d29127001001000000202010000000000010500";
const GOLDEN_SOURCE: SourceId = SourceId(SERVER_ID_BASE + 26_799);
/// A server bound to port 0 has the base id.
const PORT_ZERO_SOURCE: SourceId = SourceId(SERVER_ID_BASE);
const CLIENT: SourceId = SourceId(42);
const REQUEST_BODY: [u8; 4] = [0; 4];
const READ_TIMEOUT: Duration = Duration::from_millis(20);
const RECEIVE_ATTEMPTS: usize = 100;
const RECEIVE_BUFFER_BYTES: usize = 256;
const SUBSCRIBE_DEADLINE: Duration = Duration::from_secs(2);
/// Hex digits per byte.
const HEX_DIGITS: usize = 2;
const HEX_RADIX: u32 = 16;

#[test]
fn pad_packets_match_typescript() {
    let recentering = pad_packet(GOLDEN_SOURCE, &pad_body(&SAMPLE_PAD, 1, true)).unwrap();
    let released = pad_packet(GOLDEN_SOURCE, &pad_body(&SAMPLE_PAD, 2, false)).unwrap();
    assert_eq!(hex(&recentering), PAD_RECENTERING);
    assert_eq!(hex(&released), PAD_RELEASED);
}

fn client_for(server: &Server) -> UdpSocket {
    let client = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    client.set_read_timeout(Some(READ_TIMEOUT)).unwrap();
    client.connect(server.local_addr().unwrap()).unwrap();
    client
}

fn request(client: &UdpSocket, event: Event) {
    let packet = frame_dsu(Magic::Client, CLIENT, event, &REQUEST_BODY).unwrap();
    client.send(&packet).unwrap();
}

/// Polls the server until the client receives a datagram.
fn receive(server: &mut Server, client: &UdpSocket) -> Vec<u8> {
    let mut buffer = [0; RECEIVE_BUFFER_BYTES];
    let received = (0..RECEIVE_ATTEMPTS).find_map(|_attempt| {
        server.poll().unwrap();
        client.recv(&mut buffer).ok()
    });
    let len = received.expect("no datagram arrived");
    buffer[..len].to_vec()
}

/// Rewrites a golden datagram for another server id, so it can be compared with a live reply.
fn with_source(golden: &str, source: SourceId) -> Vec<u8> {
    let bytes: Vec<u8> = golden
        .as_bytes()
        .chunks(HEX_DIGITS)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), HEX_RADIX).unwrap())
        .collect();
    let code = u32::from_le_bytes(bytes[EVENT_RANGE].try_into().unwrap());
    let event = Event::from_code(code).unwrap();
    frame_dsu(Magic::Server, source, event, &bytes[HEADER_BYTES..]).unwrap()
}

/// Polls until the server has subscribed `count` clients or the deadline passes.
fn poll_for_clients(server: &mut Server, count: usize) {
    let deadline = Instant::now() + SUBSCRIBE_DEADLINE;
    loop {
        let is_done = server.client_count() == count || Instant::now() >= deadline;
        if is_done {
            return;
        }
        server.poll().unwrap();
    }
}

#[test]
fn answers_version_and_ports_then_streams_pad() {
    let mut server = Server::bind(0).unwrap();
    let client = client_for(&server);

    request(&client, Event::Version);
    let expected_version = with_source(VERSION_REPLY, PORT_ZERO_SOURCE);
    assert_eq!(receive(&mut server, &client), expected_version);

    request(&client, Event::Ports);
    let expected_ports = with_source(PORTS_REPLY, PORT_ZERO_SOURCE);
    assert_eq!(receive(&mut server, &client), expected_ports);

    request(&client, Event::Pad);
    poll_for_clients(&mut server, 1);
    server.recenter_until = Instant::now();
    server.send_pad(&SAMPLE_PAD).unwrap();
    let expected = pad_packet(PORT_ZERO_SOURCE, &pad_body(&SAMPLE_PAD, 1, false)).unwrap();
    assert_eq!(receive(&mut server, &client), expected);
}

#[test]
fn ignores_garbage() {
    let mut server = Server::bind(0).unwrap();
    let client = client_for(&server);
    client.send(b"DSUC short").unwrap();
    let server_packet = frame_dsu(Magic::Server, SourceId(1), Event::Pad, &[]).unwrap();
    client.send(&server_packet).unwrap();
    request(&client, Event::Version); // a valid request after the garbage proves it was read
    receive(&mut server, &client);
    assert_eq!(server.client_count(), 0);
}
