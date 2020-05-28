// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::Result;
use byteorder::{ByteOrder, LittleEndian};
use fidl_fuchsia_media::*;
use fuchsia_zircon as zx;
use std::io::Read;

const MIN_FMT_CHUNK_SIZE: u32 = 16;
const PCM_ENCODING: u16 = 1;

type Converter = fn(zx::Vmo, u64) -> Result<(zx::Vmo, u64)>;

pub struct WavReader<'a> {
    slice: &'a [u8],
}

#[derive(Debug)]
pub struct Wav {
    pub vmo: zx::Vmo,
    pub size: u64,
    pub stream_type: AudioStreamType,
}

impl WavReader<'_> {
    pub fn read(mut r: impl Read) -> Result<Wav> {
        let mut buffer = Vec::new();
        r.read_to_end(&mut buffer)?;

        (WavReader { slice: &buffer[..] }).parse_wav()
    }

    fn parse_wav(&mut self) -> Result<Wav> {
        self.parse_fourcc("RIFF")?;
        let _file_size = self.parse_u32()?;
        self.parse_fourcc("WAVE")?;

        let mut stream_type: Option<AudioStreamType> = None;
        let mut converter: Option<Converter> = None;
        let mut size: u64 = 0;
        let mut vmo: Option<zx::Vmo> = None;

        while !self.slice.is_empty() {
            match self.parse_any_fourcc()?.as_ref() {
                "fmt " => {
                    let (st, conv) = self.parse_fmt()?;
                    stream_type = Some(st);
                    converter = conv;
                }
                "data" => {
                    let (s, v) = self.parse_data()?;
                    size = s;
                    vmo = Some(v);
                }
                _name => {
                    self.parse_ignored_chunk()?;
                }
            }
        }

        let mut vmo = vmo.ok_or_else(|| anyhow::format_err!("No data chunk found"))?;
        let stream_type = stream_type.ok_or_else(|| anyhow::format_err!("No fmt chunk found"))?;

        if let Some(c) = converter {
            let (v, s) = c(vmo, size)?;
            vmo = v;
            size = s;
        }

        Ok(Wav { vmo, size, stream_type })
    }

    fn parse_fourcc(&mut self, fourcc: &str) -> Result<()> {
        let s = self
            .slice
            .get(0..4)
            .ok_or_else(|| anyhow::format_err!("Expected fourcc {}", fourcc))?;
        if fourcc.bytes().eq(s.iter().copied()) {
            self.slice = &self.slice[4..];
            Ok(())
        } else {
            Err(anyhow::format_err!("Expected fourcc {}", fourcc))
        }
    }

    fn parse_any_fourcc(&mut self) -> Result<String> {
        let s = self.slice.get(0..4).ok_or_else(|| anyhow::format_err!("Expected fourcc"))?;
        self.slice = &self.slice[4..];
        return Ok(String::from_utf8_lossy(s).into_owned());
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let s = self.slice.get(0..2).ok_or_else(|| anyhow::format_err!("Expected u16"))?;
        self.slice = &self.slice[2..];
        Ok(LittleEndian::read_u16(s))
    }

    fn parse_u32(&mut self) -> Result<u32> {
        let s = self.slice.get(0..4).ok_or_else(|| anyhow::format_err!("Expected u32"))?;
        self.slice = &self.slice[4..];
        Ok(LittleEndian::read_u32(s))
    }

    fn parse_fmt(&mut self) -> Result<(AudioStreamType, Option<Converter>)> {
        fn sample_format_from_bits_per_sample(
            bits_per_sample: u16,
        ) -> Result<(AudioSampleFormat, Option<Converter>)> {
            match bits_per_sample {
                8 => Ok((AudioSampleFormat::Unsigned8, None)),
                16 => Ok((AudioSampleFormat::Signed16, None)),
                24 => Ok((AudioSampleFormat::Signed24In32, Some(convert_24_to_32))),
                32 => Ok((AudioSampleFormat::Signed24In32, None)),
                _ => Err(anyhow::format_err!("Invalid bits per sample {}", bits_per_sample)),
            }
        }

        let chunk_size = self.parse_u32()?;
        if chunk_size < 16 {
            return Err(anyhow::format_err!("Invalid fmt chunk size {}", chunk_size));
        }

        let encoding = self.parse_u16()?;
        if encoding != PCM_ENCODING {
            return Err(anyhow::format_err!("Unrecognized audio encoding {}", encoding));
        }

        let channel_count = self.parse_u16()?;
        let frames_per_second = self.parse_u32()?;
        let _byte_rate = self.parse_u32()?;
        let _block_alignment = self.parse_u16()?;
        let bits_per_sample = self.parse_u16()?;

        if channel_count < 1 || channel_count > 2 {
            return Err(anyhow::format_err!("Invalid channel count {}", channel_count));
        }

        if chunk_size > MIN_FMT_CHUNK_SIZE {
            self.slice =
                &self.slice.get((chunk_size - MIN_FMT_CHUNK_SIZE) as usize..).ok_or_else(|| {
                    anyhow::format_err!("End of file in fmt chunk, size {}", chunk_size)
                })?;
        }

        let (sample_format, converter) = sample_format_from_bits_per_sample(bits_per_sample)?;

        return Ok((
            AudioStreamType {
                sample_format: sample_format,
                channels: channel_count as u32,
                frames_per_second: frames_per_second,
            },
            converter,
        ));
    }

    fn parse_data(&mut self) -> Result<(u64, zx::Vmo)> {
        let chunk_size = self.parse_u32()?;
        if chunk_size == 0 {
            return Err(anyhow::format_err!("Invalid data chunk size {}", chunk_size));
        }

        let vmo = zx::Vmo::create(chunk_size as u64)?;
        vmo.write(
            &self
                .slice
                .get(..chunk_size as usize)
                .ok_or_else(|| anyhow::format_err!("End of file in data chunk"))?,
            0,
        )?;

        self.slice = &self.slice[chunk_size as usize..];

        Ok((chunk_size as u64, vmo))
    }

    fn parse_ignored_chunk(&mut self) -> Result<()> {
        let chunk_size = self.parse_u32()?;
        if chunk_size == 0 {
            return Ok(());
        }

        self.slice = &self.slice.get(chunk_size as usize..).ok_or_else(|| {
            anyhow::format_err!("End of file in ignored chunk, size {}", chunk_size)
        })?;

        Ok(())
    }
}

fn convert_24_to_32(vmo: zx::Vmo, size: u64) -> Result<(zx::Vmo, u64)> {
    assert!(size % 3 == 0);
    let sample_count = size / 3;
    let out_size = 4 * sample_count;

    let mut source_buffer = vec![0u8; size as usize];
    let mut dest_buffer = vec![0u8; out_size as usize];

    vmo.read(&mut source_buffer[..], 0)?;

    let mut source_index = 0;
    let mut dest_index = 1;
    for _ in 0..sample_count {
        dest_buffer[dest_index..dest_index + 3]
            .copy_from_slice(&source_buffer[source_index..source_index + 3]);
        source_index += 3;
        dest_index += 4;
    }

    let out_vmo = zx::Vmo::create(out_size)?;
    out_vmo.write(&dest_buffer[..], 0)?;

    Ok((out_vmo, out_size))
}

#[cfg(test)]
mod test {
    use super::*;
    use fuchsia_async as fasync;
    use fuchsia_zircon as zx;
    use matches::assert_matches;

    #[fasync::run_singlethreaded]
    #[test]
    async fn successful() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x01, 0x02, 0x03, 0x04, // frames
        ];
        let wav = WavReader::read(&t0[..])?;
        assert_matches!(wav.size, 4);
        assert_matches!(
            wav.stream_type,
            AudioStreamType {
                sample_format: AudioSampleFormat::Signed16,
                channels: 2,
                frames_per_second: 44100,
            }
        );
        verify_vmo(&wav.vmo, &t0[(t0.len() - wav.size as usize)..])?;

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn long_fmt_chunk() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x14, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x00, 0x00, 0x00, 0x00, // extra fmt stuff
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x01, 0x02, 0x03, 0x04, // frames
        ];
        let wav = WavReader::read(&t0[..])?;
        assert_matches!(wav.size, 4);
        assert_matches!(
            wav.stream_type,
            AudioStreamType {
                sample_format: AudioSampleFormat::Signed16,
                channels: 2,
                frames_per_second: 44100,
            }
        );
        verify_vmo(&wav.vmo, &t0[(t0.len() - wav.size as usize)..])?;

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn no_riff_header() -> Result<()> {
        let t0 = [
            0x52, 0x41, 0x46, 0x46, // 'RAFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn no_wave() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x4f, 0x56, 0x45, // 'WOVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn no_fmt_chunk() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x74, // 'fmtt'
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn short_fmt_chunk() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x0e, 0x00, 0x00, 0x00, // fmt chunk size (14 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn bad_encoding() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x02, 0x00, // encoding (2 = bad)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn zero_channels() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x00, 0x00, // channel count (0)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn three_channels() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x03, 0x00, // channel count (3)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn no_data_chunk() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x65, // 'date'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];
        let _ = WavReader::read(&t0[..]).unwrap_err();

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn truncated() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x10, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x10, 0xb1, 0x02, 0x00, // byte rate (44100 * 4)
            0x04, 0x00, // block alignment (4)
            0x10, 0x00, // bits/sample (16)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x04, 0x00, 0x00, 0x00, // data chunk size (4)
            0x00, 0x00, 0x00, 0x00, // frames
        ];

        for i in 1..44 {
            let _ = WavReader::read(&t0[..i]).unwrap_err();
        }

        Ok(())
    }

    #[fasync::run_singlethreaded]
    #[test]
    async fn packed24() -> Result<()> {
        let t0 = [
            0x52, 0x49, 0x46, 0x46, // 'RIFF'
            0x00, 0x00, 0x00, 0x00, // RIFF chunk size (ignored)
            0x57, 0x41, 0x56, 0x45, // 'WAVE'
            0x66, 0x6d, 0x74, 0x20, // 'fmt '
            0x12, 0x00, 0x00, 0x00, // fmt chunk size (16 bytes)
            0x01, 0x00, // encoding (1 = pcm)
            0x02, 0x00, // channel count (2)
            0x44, 0xac, 0x00, 0x00, // frames/second (44100)
            0x98, 0x09, 0x04, 0x00, // byte rate (44100 * 6)
            0x06, 0x00, // block alignment (6)
            0x18, 0x00, // bits/sample (24)
            0x00, 0x00, // extra param size (0)
            0x64, 0x61, 0x74, 0x61, // 'data'
            0x0c, 0x00, 0x00, 0x00, // data chunk size (12)
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // frame
            0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, // frame
        ];
        let wav = WavReader::read(&t0[..])?;
        assert_matches!(wav.size, 16);
        assert_matches!(
            wav.stream_type,
            AudioStreamType {
                sample_format: AudioSampleFormat::Signed24In32,
                channels: 2,
                frames_per_second: 44100,
            }
        );
        let expected = [
            0x00, 0x01, 0x02, 0x03, 0x00, 0x04, 0x05, 0x06, 0x00, 0x07, 0x08, 0x09, 0x00, 0x0a,
            0x0b, 0x0c,
        ];
        verify_vmo(&wav.vmo, &expected)?;

        Ok(())
    }

    fn verify_vmo(vmo: &zx::Vmo, expected: &[u8]) -> Result<()> {
        let mut buffer = std::vec::Vec::with_capacity(expected.len());
        buffer.resize(expected.len(), 0);
        vmo.read(&mut buffer[..], 0)?;
        assert_matches!(buffer == expected, true);
        Ok(())
    }
}
