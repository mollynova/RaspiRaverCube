use std::f64;
use hound;

fn main() {
  pub const PI: f64 = 3.14159265358979323846264338327950288f64;
  let mut bits: Vec<_> = Vec::new();
  let mut fin: Vec<_> = Vec::new();
  let mark_sample_rate: f64 = 2225.0;
  let mark_target_freq: f64 = 1270.0;
  let space_sample_rate: f64 = 2025.0;
  let space_target_freq: f64 = 1070.0;
  let block_size: f64 = 1.8;
  let k_space = 0.5 + f64::from((block_size * space_target_freq)/space_sample_rate);
  let k_mark = 0.5 + f64::from((block_size * mark_target_freq)/mark_sample_rate);
  let w_space = ((2.0 * PI)/block_size) * k_space;
  let w_mark = ((2.0 * PI)/ block_size) * k_mark;
  let cosine_space = w_space.cos();
  let cosine_mark = w_mark.cos();
  let sine_space = w_space.sin();
  let sine_mark = w_mark.sin();
  let coeff_space = 2.0 * cosine_space;
  let coeff_mark = 2.0 * cosine_mark;

  let mut reader = hound::WavReader::open("novash.wav").unwrap();
  let samples: Vec<_> = reader.samples::<i16>()
                        .map(|s| f64::from(s.unwrap()) / f64::from(std::i16::MAX)).collect();

  let chunked_samples: Vec<_> = samples.chunks_exact(160).collect();
  let (mut mq0, mut mq1, mut mq2, mut sq0, mut sq1, mut sq2) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
  for chunk in chunked_samples {
    // chunk here is each block of 160 samples
    for samp in chunk {
      mq0 = coeff_mark * mq1 - mq2 + samp;
      mq2 = mq1;
      mq1 = mq0;
      sq0 = coeff_space * sq1 - sq2 + samp;
      sq2 = sq1;
      sq1 = sq0;
    }
    let m_real = mq1 - mq2 * cosine_mark;
    let m_imag = mq2 * sine_mark;
    let m_mag_squared = (m_real * m_real) + (m_imag * m_imag);
    let m_mag = m_mag_squared.sqrt();
    let s_real = sq1 - sq2 * cosine_space;
    let s_imag = sq2 * sine_space;
    let s_mag_squared = (s_real * s_real) + (s_imag * s_imag);
    let s_mag = s_mag_squared.sqrt();
    // HERE. if s_mag > m_mag, it's a 0. if m_mag > s_mag, it's a 1.k
    if s_mag > m_mag {
      bits.push(0);
    } else {
      bits.push(1);
    }
}
