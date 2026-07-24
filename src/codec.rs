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
use crate::opus;
use crate::opus::OpusEncoder;
use std::ffi::c_int;

#[derive(Debug)]
pub(crate) struct Encoder {
    encoder: *mut OpusEncoder,
    frame_size: usize,
}

// The Opus encoder is a self-contained context that is only ever touched from a
// single thread (the output thread), so it is safe to move across threads.
unsafe impl Send for Encoder {}

impl Encoder {
    pub(crate) fn new(
        bitrate: u32,
        frame_duration: u32,
        sample_rate: u32,
        channels: u8,
    ) -> Result<Self> {
        let mut error: i32 = 0;

        let encoder = unsafe {
            opus::opus_encoder_create(
                sample_rate as i32,
                channels as i32,
                opus::OPUS_APPLICATION_AUDIO as c_int,
                &mut error,
            )
        };
        if encoder.is_null() {
            return Err(Error::Codec("opus_encoder_create failed".to_string()));
        }

        unsafe {
            opus::opus_encoder_ctl(
                encoder,
                opus::OPUS_SET_BITRATE_REQUEST as c_int,
                bitrate as c_int,
            );
        }

        Ok(Self {
            encoder,
            frame_size: ((sample_rate as usize) / 1000)
                * (frame_duration as usize)
                * (channels as usize),
        })
    }

    /// Number of interleaved samples that make up one codec frame, derived from
    /// the sample rate, frame duration and channel count.
    pub(crate) fn frame_size(&self) -> usize {
        self.frame_size
    }

    pub(crate) fn encode(&mut self, pcm: &[i16]) -> Result<Vec<u8>> {
        if pcm.len() != self.frame_size {
            return Err(Error::Codec(format!(
                "Frame length {} != {}",
                pcm.len(),
                self.frame_size
            )));
        }

        let mut buffer = [0u8; 4000];

        let bytes: usize = unsafe {
            opus::opus_encode(
                self.encoder,
                pcm.as_ptr(),
                pcm.len() as c_int,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            )
        } as usize;

        Ok(buffer[0..bytes].to_vec())
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            opus::opus_encoder_destroy(self.encoder);
        }
    }
}
