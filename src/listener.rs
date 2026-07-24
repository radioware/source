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

use std::net::UdpSocket;
use std::sync::mpsc::Sender;

pub(crate) struct Listener {
    pcm_sender: Sender<f64>,
    sck: UdpSocket,
}

impl Listener {
    pub(crate) fn new(pcm_sender: Sender<f64>, bind_host: &str, bind_port: u16) -> Self {
        let bind = format!("{}:{}", bind_host, bind_port);
        let sck = UdpSocket::bind(bind).unwrap();

        Self { pcm_sender, sck }
    }
}
