/*
 * Radioware Source
 * Copyright (C) 2026 Luca Cireddu <sardylan@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see <https://www.gnu.org/licenses/>.
 *
 */

use mio::{Events, Interest, Poll};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use tracing::info;

pub(crate) fn run(
    mut poll: Poll,
    socket_bind: &str,
    pcm_sender: Sender<f32>,
) -> crate::errors::Result<()> {
    info!("Creating input socket");
    let sck = UdpSocket::bind(socket_bind)?;
    sck.set_nonblocking(true)?;
    let mut sck = mio::net::UdpSocket::from_std(sck);

    info!("Registering socket on poller");
    poll.registry()
        .register(&mut sck, crate::common::UDP_TOKEN, Interest::READABLE)?;

    info!("Creating buffers");
    let mut events = Events::with_capacity(8);
    let mut buffer = [0u8; 8192];
    let mut fifo: Vec<u8> = Vec::new();

    info!("Starting input loop");
    loop {
        if let Err(e) = poll.poll(&mut events, None) {
            if e.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(e.into());
        }

        for event in events.iter() {
            match event.token() {
                crate::common::WAKER_TOKEN => return Ok(()),
                crate::common::UDP_TOKEN => loop {
                    let ln = match sck.recv_from(&mut buffer) {
                        Ok((ln, _remote_addr)) => ln,
                        Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                        Err(e) => return Err(e.into()),
                    };
                    fifo.extend_from_slice(&buffer[..ln]);

                    let usable = fifo.len() - (fifo.len() % 4);
                    fifo[..usable]
                        .chunks_exact(4)
                        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                        .try_for_each(|sample| pcm_sender.send(sample))?;
                    fifo.drain(..usable);
                },
                _ => unreachable!("unexpected mio token"),
            }
        }
    }
}
