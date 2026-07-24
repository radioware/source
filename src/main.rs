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

use crate::errors::Result;
use mio::{Poll, Waker};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use signal_hook::low_level;
use std::env;
use std::thread::JoinHandle;
use tracing::{Level, error, info};

mod codec;
mod common;
mod config;
mod errors;
mod input;
mod listener;
mod log;
mod opus;
mod output;
mod ui;

fn main() {
    ui::header();

    if let Err(e) = program() {
        error!("{}", e)
    }
}

fn program() -> Result<()> {
    let log_level: Level = env::var("LOG_LEVEL")
        .unwrap_or_default()
        .parse()
        .unwrap_or_else(|_| Level::WARN);
    log::configure(log_level);

    info!("Parsing configuration file");
    let configuration = config::parse()?;

    info!("Installing signal handler");
    let mut signals = Signals::new([SIGINT, SIGTERM])?;

    info!("Creating Opus encoder");
    let encoder = codec::Encoder::new(
        configuration.output_bitrate,
        configuration.frame_duration,
        configuration.input_samplerate,
        configuration.channels,
    )?;

    info!("Creating channel");
    let (pcm_sender, pcm_receiver) = std::sync::mpsc::channel::<f32>();

    info!("Creating poller");
    let poll = Poll::new()?;
    let waker = Waker::new(poll.registry(), common::WAKER_TOKEN)?;

    info!("Starting input thread");
    let input_handler: JoinHandle<Result<()>> = std::thread::spawn(move || {
        let result = input::run(poll, &configuration.input_bind, pcm_sender);
        if result.is_err() {
            let _ = low_level::raise(SIGTERM);
        }
        info!("Input thread stopped");
        result
    });

    info!("Starting output thread");
    let output_host = configuration.output_host.clone();
    let output_handler: JoinHandle<Result<()>> = std::thread::spawn(move || {
        let result = output::run(encoder, &configuration.output_bind, pcm_receiver, output_host);
        if result.is_err() {
            let _ = low_level::raise(SIGTERM);
        }
        info!("Output thread stopped");
        result
    });

    info!("Running; waiting for shutdown signal");
    if let Some(signal) = signals.forever().next() {
        info!("Received signal {}", signal);
    }

    info!("Shutting down");
    waker.wake()?;

    input_handler.join()??;
    output_handler.join()??;

    Ok(())
}
