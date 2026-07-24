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

use crate::errors::{Error, Result};
use clap::Parser;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_CONFIG_FILE: &str = "radioware.toml";
const ENV_PREFIX: &str = "RADIOWARE_";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Configuration {
    pub(crate) input_bind: String,
    pub(crate) input_samplerate: u32,
    pub(crate) channels: u8,
    pub(crate) frame_duration: u32,
    pub(crate) output_bind: String,
    pub(crate) output_host: String,
    pub(crate) output_samplerate: u32,
    pub(crate) output_bitrate: u32,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            input_bind: "[::]:8888".to_string(),
            input_samplerate: 48000,
            channels: 1,
            frame_duration: 20,
            output_bind: "[::]:0".to_string(),
            output_host: "localhost:1235".to_string(),
            output_samplerate: 8000,
            output_bitrate: 16000,
        }
    }
}

/// Radioware audio source.
///
/// Reads f32le PCM audio from a UDP socket, encodes it with Opus, and streams
/// the result to an output UDP socket.
///
/// Settings are resolved by layering, from lowest to highest precedence:
/// built-in defaults, the TOML config file, `RADIOWARE_*` environment
/// variables, and finally these command-line flags.
#[derive(Debug, Parser, Serialize)]
#[command(author, version)]
struct Cli {
    /// Path to the TOML configuration file.
    ///
    /// Values from this file override the built-in defaults but are themselves
    /// overridden by environment variables and command-line flags. Defaults to
    /// `radioware.toml` in the working directory; a missing file is ignored.
    #[arg(short, long, env = "RADIOWARE_CONFIG", value_name = "path")]
    #[serde(skip)]
    config: Option<PathBuf>,

    /// Address the input UDP socket binds to.
    ///
    /// Accepts any `host:port`. Use `[::]:8888` to listen on all IPv6 (and, on
    /// dual-stack hosts, IPv4) interfaces. Default: `[::]:8888`.
    #[arg(long, value_name = "host:port")]
    #[serde(skip_serializing_if = "Option::is_none")]
    input_bind: Option<String>,

    /// Sample rate, in Hz, of the incoming f32le PCM.
    ///
    /// Must match the rate the sender produces and be one Opus supports (8000,
    /// 12000, 16000, 24000 or 48000). Default: 48000.
    #[arg(long, value_name = "Hz")]
    #[serde(skip_serializing_if = "Option::is_none")]
    input_samplerate: Option<u32>,

    /// Number of interleaved audio channels in the input stream.
    ///
    /// 1 for mono, 2 for stereo. Together with the sample rate and frame
    /// duration this determines how many samples make up one codec frame.
    /// Default: 1.
    #[arg(long, value_name = "N")]
    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<u8>,

    /// Codec frame duration, in milliseconds.
    ///
    /// One of the Opus-supported durations: 2.5, 5, 10, 20, 40 or 60 ms. Larger
    /// frames improve compression at the cost of latency. Default: 20.
    #[arg(long, value_name = "ms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    frame_duration: Option<u32>,

    /// Address the output UDP socket binds to locally.
    ///
    /// Usually left as `[::]:0` to let the OS choose an ephemeral source port.
    /// Default: `[::]:0`.
    #[arg(long, value_name = "host:port")]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_bind: Option<String>,

    /// Destination the encoded audio is sent to.
    ///
    /// A `host:port` that receives one UDP datagram per encoded Opus frame.
    /// Default: `localhost:1235`.
    #[arg(long, value_name = "host:port")]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_host: Option<String>,

    /// Output sample rate, in Hz.
    ///
    /// The sample rate the outgoing Opus stream is encoded at. Default: 8000.
    #[arg(long, value_name = "Hz")]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_samplerate: Option<u32>,

    /// Target Opus bitrate, in bits per second.
    ///
    /// Passed to the encoder via `OPUS_SET_BITRATE`. Higher values improve
    /// quality at the cost of bandwidth. Default: 16000.
    #[arg(long, value_name = "b/s")]
    #[serde(skip_serializing_if = "Option::is_none")]
    output_bitrate: Option<u32>,
}

pub(crate) fn parse() -> Result<Configuration> {
    let cli = Cli::parse();

    let config_file = cli
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE));

    Figment::from(Serialized::defaults(Configuration::default()))
        .merge(Toml::file(config_file))
        .merge(Env::prefixed(ENV_PREFIX))
        .merge(Serialized::defaults(&cli))
        .extract()
        .map_err(|e| Error::Configuration(e.to_string()))
}
