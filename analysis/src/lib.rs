pub mod detection;

use std::{
    sync::mpsc::{channel, Receiver, Sender},
    iter::successors,
    thread,
};
use std::thread::JoinHandle;
use resonators::ResonatorBank;


pub enum AudioMessage {
    AudioChunk(Vec<f32>),
    UpdateMinFreq(f32),
    UpdateMaxFreq(f32),
    UpdateBounds(f32, f32),
    UpdateBinsPerOctave(u32),
    RequestFreqs,
    Shutdown,
}

pub enum OutputMessage {
    NewFreqFrame(Vec<f32>),
    ConfirmMinFreq(f32, bool, Option<String>),
    ConfirmMaxFreq(f32, bool, Option<String>),
    ConfirmBounds(f32, f32, bool, Option<String>),
    ConfirmBinsPerOctave(u32, bool, Option<String>),
    ReturnFreqs(Vec<f32>),
    Shutdown,
    Confirm,
}


pub fn start_process_thread(sample_rate: f32, min_f: f32, max_f: f32, bins_per_octave: u32, out_post: Sender<OutputMessage>) -> Result<(Sender<AudioMessage>, JoinHandle<Result<(), String>>), String> {
    if sample_rate.is_nan() {
        return Err(String::from("sample_rate must not be NaN"));
    } else if sample_rate.is_infinite() {
        return Err(String::from("sample_rate must not be infinite"));
    } else if sample_rate.is_subnormal() {
        return Err(String::from("sample_rate must not be subnormal"));
    } else if sample_rate <= 0.0 {
        return Err(String::from("sample_rate must be > 0"));
    } else if min_f.is_nan() {
        return Err(String::from("min_f must not be NaN"));
    } else if min_f.is_infinite() {
        return Err(String::from("min_f must not be infinite"));
    } else if min_f.is_subnormal() {
        return Err(String::from("min_f must not be subnormal"));
    } else if min_f <= 0.0 {
        return Err(String::from("min_f must be > 0"));
    } else if max_f.is_nan() {
        return Err(String::from("max_f must not be NaN"));
    } else if max_f.is_infinite() {
        return Err(String::from("max_f must not be infinite"));
    } else if max_f.is_subnormal() {
        return Err(String::from("max_f must not be subnormal"));
    } else if max_f <= min_f {
        return Err(String::from("max_f must be > min_f"));
    } else if max_f > sample_rate / 2.0 {
        return Err(String::from("max_f must be <= sample_rate / 2"));
    } else if bins_per_octave == 0 {
        return Err(String::from("bins_per_octave must be > 0"));
    } else if out_post.send(OutputMessage::Confirm).is_err() {
        return Err(String::from("out_mail must be an active Sender"))
    }

    let (in_post, in_mail) = channel::<AudioMessage>();

    let handle = thread::spawn(move || -> Result<(), String> {
        process_loop(sample_rate, min_f, max_f, bins_per_octave, in_mail, out_post)
    });

    Ok((in_post, handle))
}

fn process_loop(sample_rate: f32, min_f: f32, max_f: f32, bp_oct: u32, in_mail: Receiver<AudioMessage>, out_post: Sender<OutputMessage>) -> Result<(), String> {
    let mut min_frequency = min_f;
    let mut max_frequency = max_f;
    let mut bins_per_octave = bp_oct;
    let mut bank: ResonatorBank = ResonatorBank::from_frequencies(&get_bins(min_frequency, max_frequency, bins_per_octave), sample_rate);

    for rec in in_mail {
        match rec {
            AudioMessage::AudioChunk(data) => {
                bank.process_samples(&data);
                let result = bank.magnitudes();
                
                safe_out_msg(&out_post, OutputMessage::NewFreqFrame(result))?;
            }
            AudioMessage::UpdateMinFreq(min_freq) => {
                if min_freq.is_nan() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMinFreq(min_freq, false, Some(String::from("min_freq can not be NaN"))))?
                } else if min_freq.is_infinite() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMinFreq(min_freq, false, Some(String::from("min_freq can not be infinite"))))?
                } else if min_freq.is_subnormal() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMinFreq(min_freq, false, Some(String::from("min_freq can not be subnormal"))))?
                } else if min_freq >= max_frequency {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMinFreq(min_freq, false, Some(String::from("min_freq must be < max_frequency"))))?
                } else if min_freq <= 0.0 {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMinFreq(min_freq, false, Some(String::from("min_freq must be > 0.0"))))?
                }

                min_frequency = min_freq;

                bank = ResonatorBank::from_frequencies(&get_bins(min_frequency, max_frequency, bins_per_octave), sample_rate);
                safe_out_msg(&out_post, OutputMessage::ConfirmMinFreq(min_frequency, true, None))?;
            }
            AudioMessage::UpdateMaxFreq(max_freq) => {
                if max_freq.is_nan() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMaxFreq(max_freq, false, Some(String::from("max_freq can not be NaN"))))?
                } else if max_freq.is_infinite() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMaxFreq(max_freq, false, Some(String::from("max_freq can not be infinite"))))?
                } else if max_freq.is_subnormal() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMaxFreq(max_freq, false, Some(String::from("max_freq can not be subnormal"))))?
                } else if max_freq <= min_frequency {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMaxFreq(max_freq, false, Some(String::from("max_freq must be > min_frequency"))))?
                } else if max_freq > sample_rate / 2.0 {
                    safe_out_msg(&out_post, OutputMessage::ConfirmMaxFreq(max_freq, false, Some(String::from("max_freq must be <= sample_rate / 2"))))?
                }

                max_frequency = max_freq;

                bank = ResonatorBank::from_frequencies(&get_bins(min_frequency, max_frequency, bins_per_octave), sample_rate);
                safe_out_msg(&out_post, OutputMessage::ConfirmMaxFreq(max_frequency, true, None))?;
            }
            AudioMessage::UpdateBounds(min_freq, max_freq) => {
                if min_freq.is_nan() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("min_freq can not be NaN"))))?
                } else if min_freq.is_infinite() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("min_freq can not be infinite"))))?
                } else if min_freq.is_subnormal() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("min_freq can not be subnormal"))))?
                } else if min_freq <= 0.0 {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("min_freq must be > 0.0"))))?
                } else if max_freq.is_nan() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("max_freq can not be NaN"))))?
                } else if max_freq.is_infinite() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("max_freq can not be infinite"))))?
                } else if max_freq.is_subnormal() {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("max_freq can not be subnormal"))))?
                } else if max_freq > sample_rate / 2.0 {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("max_freq must be <= sample_rate / 2"))))?
                } else if min_freq >= max_freq {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_freq, max_freq, false, Some(String::from("min_freq must be < max_freq"))))?
                }

                min_frequency = min_freq;
                max_frequency = max_freq;

                bank = ResonatorBank::from_frequencies(&get_bins(min_frequency, max_frequency, bins_per_octave), sample_rate);
                safe_out_msg(&out_post, OutputMessage::ConfirmBounds(min_frequency, max_frequency, true, None))?;
            }
            AudioMessage::UpdateBinsPerOctave(bins_p_octave) => {
                if bins_p_octave == 0 {
                    safe_out_msg(&out_post, OutputMessage::ConfirmBinsPerOctave(bins_p_octave, false, Some(String::from("bins_p_octave must be > 0"))))?
                }

                bins_per_octave = bins_p_octave;

                bank = ResonatorBank::from_frequencies(&get_bins(min_frequency, max_frequency, bins_per_octave), sample_rate);
                safe_out_msg(&out_post, OutputMessage::ConfirmBinsPerOctave(bins_per_octave, true, None))?;
            }
            AudioMessage::RequestFreqs => {
                safe_out_msg(&out_post, OutputMessage::ReturnFreqs(bank.frequencies()))?;
            }
            AudioMessage::Shutdown => {
                safe_out_msg(&out_post, OutputMessage::Shutdown)?;
                
                return Ok(());
            }
        }
    }
    println!("Process Thread Done");
    Ok(())
}

pub fn safe_out_msg(post: &Sender<OutputMessage>, msg: OutputMessage) -> Result<(), String> {
    if let Err(_send_err) = post.send(msg) {
        eprintln!("Frontend disconnected, processing thread shutting down");
        return Err(String::from("Frontend disconnected, processing thread shutting down"))
    }
    Ok(())
}

pub fn get_bins(min_freq: f32, max_freq: f32, bins_per_octave: u32) -> Vec<f32> {
    let mult = (2.0f32).powf(1.0 / bins_per_octave as f32);

    successors(Some(min_freq), |&prev| {
        let next = prev * mult;
        if next < max_freq { Some(next) } else { None }
    }).collect()
}