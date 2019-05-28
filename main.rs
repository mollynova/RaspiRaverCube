use std::f64;
use hound;

// Typically, frequencies in music fall in the range of 30Hz ~ 3500Hz. This would be the low end of bass versus
// the higher end of a violin. I have 4 colors in my LED cube, Red Blue Green Yellow. So I'm going to divide this
// estimated range into 4 quadrants, and assign each quadrant one of the four colors. Then, I'm going to utilize
// the Goertzel algorithm. What the Goertzel algorithm will return is the -likelihood- that a sample is close to
// a specific frequency. I'm going to run each sample against the Goertzel algorithm four times- for 4 frequencies-
// each of which will be the middle frequency of each of the four quadrants. Whichever one of these 4 trials returns
// the highest result will be the one which most closely matches the frequency of the sample, and I will have it
// light up the appropriate colors on the LED cube. 

// Computing color ranges: 3500 - 30 = 3470. 3470 / 4 ~= 867. 867 / 2 ~= 434 + 30 = 464. 464 + 867 = 1331. 
// 1331 + 867 = 2198. 2198 + 867 = 3065. 

// Target frequencies:
// Red:    464 Hz
// Yellow: 1331 Hz
// Green:  2198 Hz
// Blue:   3065 Hz 

// I'm going to use a sample rate of 44100 Hz, which is standard for most music files. I will of course make sure
// that the wav files I'm feeding to the program are 44100Hz.

fn main() {
  // define PI const
  pub const PI: f64 = 3.14159265358979323846264338327950288f64;

  // sample rate for all wav files that will be used as input
  let sample_rate: f64 = 44100.0;

  // target frequency by color
  let red_freq: f64 = 464.0;
  let yellow_freq: f64 = 1331.0;
  let green_freq: f64 = 2198.0;
  let blue_freq: f64 = 3065;

  // below: all values necessary to compute magnitude, by color 
  let block_size: f64 = 1.8;

  let k_red    = 0.5 + f64::from((block_size * red_freq)/sample_rate);
  let k_yellow = 0.5 + f64::from((block_size * yellow_freq)/sample_rate);
  let k_green  = 0.5 + f64::from((block_size * green_freq)/sample_rate);
  let k_blue   = 0.5 + f64::from((block_size * blue_freq)/sample_rate);

  let w_red    = ((2.0 * PI)/block_size) * k_red;
  let w_yellow = ((2.0 * PI)/block_size) * k_yellow;
  let w_green  = ((2.0 * PI)/block_size) * k_green;
  let w_blue   = ((2.0 * PI)/block_size) * k_blue;

  let cosine_red    = w_red.cos();
  let cosine_yellow = w_yellow.cos();
  let cosine_green  = w_green.cos();
  let cosine_blue   = w_blue.cos();

  let sine_red    = w_red.sin();
  let sine_yellow = w_yellow.sin();
  let sine_green  = w_green.sin();
  let sine_blue   = w_blue.sin();

  let coeff_red    = 2.0 * cosine_red;
  let coeff_yellow = 2.0 * cosine_yellow;
  let coeff_green  = 2.0 * cosine_green;
  let coeff_blue   = 2.0 * cosine_blue;

  // use 'hound' crate to read all samples from wav file into a vector as i16 type
  let mut reader = hound::WavReader::open("file.wav").unwrap();
  let samples: Vec<_> = reader.samples::<i16>()
                        .map(|s| f64::from(s.unwrap()) / f64::from(std::i16::MAX)).collect();

  // compute the number of samples that there are per second of music for the given wav file
  let arr_length         = samples.length();
  let seconds            = 2400; // say each song is about 4 minutes
  let samples_per_second = arr_length / seconds;

  // break the vector of samples into a vector of vectors of samples, each of size 'samples_per_second'
  // The idea is that there are -lots- of "samples" in a given wav file- probably hundreds of thousands for a file the 
  // length of a typical song. If we were to examine every sample, our hardware could never keep up with lights changing
  // that quickly, and even if it could our eyes certainly couldn't. So, we figure out how many samples are in the file 
  // for each second of music, so that we can just pick one of them to play per second instead of trying to play them all.
  let chunked_samples: Vec<_> = samples.chunks_exact(samples_per_second).collect();

  // initialize more variables for the algorithm (ongoing) to compute magnitude
  let (mut red_q0, mut red_q1, mut red_q2, mut yel_q0, mut yel_q1, mut yel_q2)          = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
  let (mut green_q0, mut green_q1, mut green_q2, mut blue_q0, mut blue_q1, mut blue_q2) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

  // for each vector of samples in our vector of vectors (there are 'seconds' number of them, and each is size 'samples per second'
  for chunk in chunked_samples {
    // just check the first sample in each chunk. this way we're checking one sample per second of music
    // it can be that arbitrary. I could have checked the last one, or the middle one, or the 37th one, as long as I'm being consistent
    // it doesn't really matter
    red_q0 = coeff_red * red_q1 - red_q2 + chunk[0];
    red_q2 = red_q1;
    red_q1 = red_q0;

    yel_q0 = coeff_yellow * yel_q1 - yel_q2 + chunk[0];
    yel_q2 = yel_q1;
    yel_q1 = yel_q0;

    green_q0 = coeff_green * green_q1 - green_q2 + chunk[0];
    green_q2 = green_q1;
    green_q1 = green_q0;

    blue_q0 = coeff_blue * blue_q1 - blue_q2 + chunk[0];
    blue_q2 = blue_q1;
    blue_q1 = blue_q0;
    
    let r_real   = red_q1 - red_q2 * cosine_red;
    let r_imag   = red_q2 * sine_red;
    let r_mag_sq = (r_real * r_real) + (r_imag * r_imag);
    let r_mag    = r_mag_sq.sqrt();

    let y_real   = yel_q1 - yel_q2 * cosine_yellow;
    let y_imag   = yel_q2 * sine_yellow;
    let y_mag_sq = (y_real * y_real) + (y_imag * y_imag);
    let y_mag    = y_mag_sq.sqrt();

    let g_real   = green_q1 - green_q2 * cosine_green;
    let g_imag   = green_q2 * sine_green;
    let g_mag_sq = (g_real * g_real) + (g_imag * g_imag);
    let g_mag    = g_mag_sq.sqrt();

    let b_real   = blue_q1 - blue_q2 * cosine_blue;
    let b_imag   = blue_q2 * sine_blue;
    let b_mag_sq = (b_real * b_real) + (b_imag * b_imag);
    let b_mag    = b_mag_sq.sqrt();

    // now I have my magnitude values and can check to see which is the greatest, i.e. which 
    // color lights I should turn on on my LED cube
    
    /*
      Note: from here I'll need to use a GPIO crate for Rust so that I can communicate with my LED cube and tell it
      which lights need to come on. Beyond just turning the lights on, I need to of course tell them how long to stay
      on (one second, of course, so the light doesn't ever go fully dark while the song is playing) and tell them to
      shut off after that time has elapsed. I might have them stay on for 1.1-1.2 seconds to give it a laggy effect
      and see if it looks cool. 
      
      I have not included this code because I have yet to finish building the cube (it'll be done this week). Until the 
      cube is done there's not really any way for me to test my code for interacting with it, so I'll leave my
      experimentation out of here as it is likely erroneous and will see heavy modifications. 
    */

    if r_mag > y_mag && r_mag > g_mag && r_mag > b_mag {
       // light up the red lightU
    } else if y_mag > r_mag && y_mag > g_mag && y_mag > b_mag {
       // light up the yellow lights
    } else if g_mag > r_mag && g_mag > y_mag && g_mag > b_mag {
       // light up the green lights
    } else {
       // light up the blue lights
       // note: this will also catch any case where the Goertzel algo returns the same likelihood
       // that a sample is multiple frequencies, which is really unlikely but possible if
       // the sample happens to be exactly between two of the target frequencies
    }
  }
}
