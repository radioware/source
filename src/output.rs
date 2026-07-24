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

use crate::codec;
use std::net::UdpSocket;
use std::sync::mpsc::Receiver;
use tracing::info;

pub(crate) fn run(
    mut encoder: codec::Encoder,
    socket_bind: &str,
    pcm_receiver: Receiver<f32>,
    output_host: String,
) -> crate::errors::Result<()> {
    let frame_size = encoder.frame_size();
    let mut frame: Vec<i16> = Vec::with_capacity(frame_size);

    info!("Creating output socket");
    let sck = UdpSocket::bind(socket_bind)?;

    info!("Starting output loop");
    while let Ok(sample) = pcm_receiver.recv() {
        let sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        frame.push(sample);

        if frame.len() == frame_size {
            let encoded = encoder.encode(&frame)?;
            sck.send_to(&encoded, &output_host)?;
            frame.clear();
        }
    }

    Ok(())
}
