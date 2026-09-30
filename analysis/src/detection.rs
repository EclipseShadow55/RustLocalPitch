use std::cmp::min;
use std::iter::{once, repeat_n, zip};

pub enum ErrorTypes {
    StandardError,
    BreakError(String),
}

pub struct FrameContext<'a> {
    pub frame_data: &'a [f32],
    pub bins: &'a [f32],
    pub bins_per_octave: usize,
}

pub fn sub_harmonic_summing(
    frame_context: FrameContext,
    num_harmonics: u8,
    harmonic_decay: f32,
    comp_turn_point: f32,
    low_favor: bool,
    peak_threshold: f32,
) -> Result<usize, ErrorTypes> {
    if frame_context.frame_data.is_empty() {
        return Err(ErrorTypes::BreakError(String::from("frame_data is empty")));
    } else if frame_context
        .frame_data
        .iter()
        .any(|&x| !(x.is_normal() || x.is_subnormal() || x == 0.0))
    {
        return Err(ErrorTypes::BreakError(String::from(
            "all elements of frame_data must not be NaN or Infinity",
        )));
    } else if !(harmonic_decay.is_normal() || harmonic_decay == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "harmonic_decay must not be Subnormal, NaN, or Infinity",
        )));
    } else if !(0.0..=1.0).contains(&harmonic_decay) {
        return Err(ErrorTypes::BreakError(String::from(
            "harmonic_decay must be in the range [0, 1]",
        )));
    } else if !(comp_turn_point.is_normal() || comp_turn_point == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "comp_turn_point must not be Subnormal, NaN, or Infinity",
        )));
    } else if comp_turn_point <= 0.0 {
        return Err(ErrorTypes::BreakError(String::from(
            "comp_turn_point must be > 0",
        )));
    } else if !(peak_threshold.is_normal() || peak_threshold == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "peak_threshold must not be Subnormal, NaN, or Infinity",
        )));
    } else if peak_threshold < 0.0 {
        return Err(ErrorTypes::BreakError(String::from(
            "peak_threshold must be >= 0",
        )));
    }

    let frame_data = frame_context.frame_data;
    let bins_per_octave = frame_context.bins_per_octave;

    let log_frame_data: Vec<f32> = frame_data
        .iter()
        .map(|x| (1.0 + comp_turn_point * x).ln())
        .collect();
    let mut result_energy = log_frame_data.clone();

    let max_shift = (log_frame_data.len() / bins_per_octave).pow(2) as u8;

    for harmonic in 2..min(2 + num_harmonics, max_shift + 1) {
        let shift_inds = (bins_per_octave as f32 * (harmonic as f32).log2()).round() as usize;
        let shifted_data = log_frame_data[shift_inds..].to_vec();
        let comp_zeros: Vec<f32> = repeat_n(0.0f32, shift_inds).collect();
        let add_set: Vec<f32> = shifted_data.iter().chain(&comp_zeros).copied().collect();

        let harm_mult = harmonic_decay.powf((harmonic - 1) as f32);
        result_energy
            .iter_mut()
            .zip(add_set.iter())
            .for_each(|(base, add)| *base += harm_mult * add);
    }

    if low_favor {
        let max_energy = *result_energy.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
        let energy_threshold = max_energy * peak_threshold;

        let padded_energy: Vec<f32> = once(0.0).chain(result_energy).chain(once(0.0)).collect();
        let above_threshold: Vec<usize> = padded_energy
            .windows(3)
            .enumerate()
            .filter(|(_, window)| {
                window[1] > window[0] && window[1] > window[2] && window[1] >= energy_threshold
            })
            .map(|(ind, _)| ind)
            .collect();

        Ok(above_threshold[0])
    } else {
        let (max_ind, _) = result_energy
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .unwrap();

        Ok(max_ind)
    }
}

pub fn peak_picking(frame_context: FrameContext) -> Result<usize, ErrorTypes> {
    if frame_context.frame_data.is_empty() {
        return Err(ErrorTypes::BreakError(String::from("frame_data is empty")));
    } else if frame_context
        .frame_data
        .iter()
        .any(|&x| !(x.is_normal() || x.is_subnormal() || x == 0.0))
    {
        return Err(ErrorTypes::BreakError(String::from(
            "all elements of frame_data must not be NaN or Infinity",
        )));
    }

    let frame_data = frame_context.frame_data;

    let (max_ind, _) = frame_data
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap();

    Ok(max_ind)
}

pub fn two_way_mismatch(
    frame_context: FrameContext,
    predicted_harms: usize,
    measured_peaks: usize,
    freq_penalty: f32,
    amp_weight: f32,
    freq_weight: f32,
    error_ratio: f32,
) -> Result<usize, ErrorTypes> {
    if frame_context
        .frame_data
        .iter()
        .any(|&x| !(x.is_normal() || x.is_subnormal() || x == 0.0))
    {
        return Err(ErrorTypes::BreakError(String::from(
            "all elements of frame_data must not be NaN or Infinity",
        )));
    } else if frame_context
        .bins
        .iter()
        .any(|&x| !(x.is_normal() || x.is_subnormal() || x == 0.0))
    {
        return Err(ErrorTypes::BreakError(String::from(
            "all elements of bins must not be NaN or Infinity",
        )));
    } else if frame_context.bins[0]
        > frame_context.bins[frame_context.bins.len() - 1] / predicted_harms as f32
    {
        return Err(ErrorTypes::BreakError(String::from(
            "predicted_harms must not exceed given bin range",
        )));
    } else if predicted_harms == 0 {
        return Err(ErrorTypes::BreakError(String::from(
            "predicted_harms must be >= 1",
        )));
    } else if measured_peaks == 0 {
        return Err(ErrorTypes::BreakError(String::from(
            "measured_peaks must be >= 1",
        )));
    } else if !(freq_penalty.is_normal() || freq_penalty == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "freq_penalty must not be Subnormal, NaN, or Infinity",
        )));
    } else if freq_penalty < 0.0 {
        return Err(ErrorTypes::BreakError(String::from(
            "freq_penalty must be >= 0",
        )));
    } else if !(amp_weight.is_normal() || amp_weight == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "amp_weight must not be Subnormal, NaN, or Infinity",
        )));
    } else if amp_weight <= 0.0 {
        return Err(ErrorTypes::BreakError(String::from(
            "amp_weight must be > 0",
        )));
    } else if !(freq_weight.is_normal() || freq_weight == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "freq_weight must not be Subnormal, NaN, or Infinity",
        )));
    } else if freq_weight <= 0.0 {
        return Err(ErrorTypes::BreakError(String::from(
            "freq_weight must be > 0",
        )));
    } else if !(error_ratio.is_normal() || error_ratio == 0.0) {
        return Err(ErrorTypes::BreakError(String::from(
            "error_ratio must not be Subnormal, NaN, or Infinity",
        )));
    } else if !(0.0..=1.0).contains(&error_ratio) {
        return Err(ErrorTypes::BreakError(String::from(
            "error_ratio must be in the range [0, 1]",
        )));
    }

    let frame_data = frame_context.frame_data;
    let bins = frame_context.bins;
    let bins_per_octave = frame_context.bins_per_octave;

    let max_freq = bins[bins.len() - 1] / predicted_harms as f32;
    let max_amp = frame_data.iter().max_by(|&a, &b| a.total_cmp(b)).unwrap();

    let candidate_bins: Vec<usize> = bins
        .iter()
        .filter(|&&item| item <= max_freq)
        .enumerate()
        .map(|(i, _)| i)
        .collect();
    let mut measured_bins = n_peaks(frame_data, measured_peaks);
    if measured_bins.is_empty() {
        return Err(ErrorTypes::StandardError);
    }
    measured_bins.sort_unstable();
    let measured_bins = measured_bins;

    let pm_errors: Vec<f32> = candidate_bins
        .iter()
        .map(|&item| {
            let pm_freq_errors: Vec<(usize, usize, f32)> =
                (1..=predicted_harms) // (harmonic ind, nearest peak ind, freq error)
                    .map(|harm| {
                        item + (bins_per_octave as f32 * (harm as f32).log2()).round() as usize
                    })
                    .map(|harm_ind| (harm_ind, bins[harm_ind]))
                    .map(|(harm_ind, harm_freq)| {
                        let (res_ind, res_freq) = measured_bins
                            .iter()
                            .map(|&meas_bin| (meas_bin, bins[meas_bin]))
                            .min_by(|&(_, a), &(_, b)| {
                                (a - harm_freq).abs().total_cmp(&(b - harm_freq).abs())
                            })
                            .unwrap();
                        (harm_ind, res_ind, (res_freq - harm_freq).abs())
                    })
                    .collect();

            let pm_total_errors: Vec<f32> = pm_freq_errors
                .iter()
                .map(|&(harm_ind, peak_ind, freq_error)| {
                    let freq_base = freq_error * bins[harm_ind].powf(-freq_penalty);
                    let amp_mult = frame_data[peak_ind] / max_amp;
                    let freq_term = amp_weight + freq_weight * (freq_error / bins[harm_ind]);

                    freq_base + amp_mult * freq_term
                })
                .collect();

            pm_total_errors.iter().sum()
        })
        .collect();

    let mp_errors: Vec<f32> = candidate_bins
        .iter()
        .map(|&item| {
            let harm_bins: Vec<(usize, f32)> = (1..=predicted_harms)
                .map(|harm| item + (bins_per_octave as f32 * (harm as f32).log2()).round() as usize)
                .map(|harm_ind| (harm_ind, bins[harm_ind]))
                .collect();

            let mp_freq_errors: Vec<(usize, usize, f32)> = measured_bins
                .iter()
                .map(|&meas_ind| (meas_ind, bins[meas_ind]))
                .map(|(meas_ind, meas_freq)| {
                    let &(res_ind, res_freq) = harm_bins
                        .iter()
                        .min_by(|&(_, a), &(_, b)| {
                            (a - meas_freq).abs().total_cmp(&(b - meas_freq).abs())
                        })
                        .unwrap();
                    (meas_ind, res_ind, (res_freq - meas_freq).abs())
                })
                .collect();

            let mp_total_errors: Vec<f32> = mp_freq_errors
                .iter()
                .map(|&(peak_ind, _, freq_error)| {
                    let freq_base = freq_error * bins[peak_ind].powf(-freq_penalty);
                    let amp_mult = frame_data[peak_ind] / max_amp;
                    let freq_term = amp_weight + freq_weight * (freq_error / bins[peak_ind]);

                    freq_base + amp_mult * freq_term
                })
                .collect();

            mp_total_errors.iter().sum()
        })
        .collect();

    let total_errors: Vec<f32> = zip(pm_errors, mp_errors)
        .map(|(a, b)| a * error_ratio + b * (1.0 - error_ratio))
        .collect();

    let (min_ind, _) = total_errors
        .iter()
        .enumerate()
        .min_by(|&(_, a), &(_, b)| a.total_cmp(b))
        .unwrap();

    Ok(min_ind)
}

fn n_peaks<T: PartialOrd + Clone>(slice: &[T], n: usize) -> Vec<usize> {
    if n == 0 || slice.is_empty() {
        return Vec::new();
    }

    let mut largest = Vec::<(usize, T)>::with_capacity(n);

    for (i, window) in slice.windows(3).enumerate() {
        if window[0] < window[1] && window[1] > window[2] {
            let pos = largest.binary_search_by(|(_, v)| {
                v.partial_cmp(&window[1])
                    .map(|s| s.reverse())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let idx = pos.unwrap_or_else(|i| i);

            if idx < n {
                largest.insert(idx, (i, window[1].clone()));
                if largest.len() > n {
                    largest.pop();
                }
            }
        }
    }

    largest.iter().map(|&(i, _)| i).collect()
}
