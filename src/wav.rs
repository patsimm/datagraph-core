use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub fn write_wav(filename: &str, samples: &[i16], sample_rate: u32) {
    let file = File::create(filename).unwrap();
    let mut w = BufWriter::new(file);

    let num_samples = samples.len() as u32;
    let data_size = num_samples * 2; // 2 bytes per i16 sample
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;

    // RIFF header
    w.write_all(b"RIFF").unwrap();
    w.write_all(&(36 + data_size).to_le_bytes()).unwrap();
    w.write_all(b"WAVE").unwrap();

    // fmt chunk
    w.write_all(b"fmt ").unwrap();
    w.write_all(&16u32.to_le_bytes()).unwrap(); // chunk size
    w.write_all(&1u16.to_le_bytes()).unwrap(); // PCM format
    w.write_all(&channels.to_le_bytes()).unwrap();
    w.write_all(&sample_rate.to_le_bytes()).unwrap();
    w.write_all(&byte_rate.to_le_bytes()).unwrap();
    w.write_all(&block_align.to_le_bytes()).unwrap();
    w.write_all(&bits_per_sample.to_le_bytes()).unwrap();

    // data chunk
    w.write_all(b"data").unwrap();
    w.write_all(&data_size.to_le_bytes()).unwrap();
    for &s in samples {
        w.write_all(&s.to_le_bytes()).unwrap();
    }
}
