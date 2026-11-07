//! Procedural tone synthesizer for MotionArcade sound effects.
//!
//! Generates real-time synthetic PCM audio samples (sine waves, harmonic chimes,
//! frequency sweeps, arpeggios, and vibrato wobbles) directly in memory, eliminating
//! any need for external audio asset files on disk.

use crate::SoundEffect;

/// Generates interleaved stereo f32 PCM audio samples for the specified `SoundEffect`.
///
/// Output format: Interleaved stereo `[L_0, R_0, L_1, R_1, ...]`.
/// If `sample_rate == 0`, returns an empty vector safely.
pub fn generate_samples(effect: &SoundEffect, sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    match effect {
        SoundEffect::DeflectChime { pan } | SoundEffect::DeflectionChime { pan } => {
            generate_deflect_chime(*pan, sample_rate)
        }
        SoundEffect::BananaChomp => generate_banana_chomp(sample_rate),
        SoundEffect::CorrectFanfare => generate_correct_fanfare(sample_rate),
        SoundEffect::GentleWobble => generate_gentle_wobble(sample_rate),
    }
}

/// Generates a metallic / crystal deflection chime with constant-power stereo panning.
///
/// - `pan`: Position from -1.0 (hard left) to +1.0 (hard right).
/// - Duration: ~0.35 seconds.
/// - Frequency: Base A5 (880 Hz) with overtone harmonic bells.
pub fn generate_deflect_chime(pan: f32, sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let pan_clamped = pan.clamp(-1.0, 1.0);
    // Constant-power panning: angle 0 = hard left, PI/4 = center, PI/2 = hard right
    let angle = (pan_clamped + 1.0) * std::f32::consts::FRAC_PI_4;
    let left_gain = angle.cos();
    let right_gain = angle.sin();

    let duration_secs = 0.35_f32;
    let num_frames = (duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_frames * 2);

    let f0 = 880.0_f32;
    let two_pi = 2.0 * std::f32::consts::PI;

    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;
        // 5ms attack to avoid clicking, followed by exponential decay
        let attack = (t / 0.005).min(1.0);
        let decay = (-t / 0.07).exp();
        let env = attack * decay;

        // Overtone series for a rich metallic chime
        let s1 = (two_pi * f0 * t).sin() * 0.55;
        let s2 = (two_pi * f0 * 2.0 * t).sin() * 0.25;
        let s3 = (two_pi * f0 * 3.01 * t).sin() * 0.15;
        let s4 = (two_pi * f0 * 4.15 * t).sin() * 0.05;

        let mono = (s1 + s2 + s3 + s4) * env * 0.75;
        samples.push(mono * left_gain);
        samples.push(mono * right_gain);
    }
    samples
}

/// Generates a cartoon banana chomp sound effect.
///
/// - Duration: ~0.18 seconds.
/// - Downward frequency sweep from 550 Hz to 130 Hz with a snappy bite attack and crunch peak.
pub fn generate_banana_chomp(sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let duration_secs = 0.18_f32;
    let num_frames = (duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_frames * 2);

    let two_pi = 2.0 * std::f32::consts::PI;
    let dt = 1.0 / sample_rate as f32;
    let mut phase = 0.0_f32;
    let center_gain = std::f32::consts::FRAC_1_SQRT_2;

    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;
        let progress = (t / duration_secs).min(1.0);

        // Quadratic downward frequency sweep: 550 Hz -> 130 Hz
        let freq = 130.0 + (550.0 - 130.0) * (1.0 - progress).powi(2);
        phase += two_pi * freq * dt;
        if phase > two_pi {
            phase -= two_pi;
        }

        // Dual-stage envelope: sharp initial bite followed by a secondary crunch pop
        let attack = (t / 0.006).min(1.0);
        let bite1 = (-t / 0.035).exp() * 0.7;
        let t_crunch = (t - 0.045).max(0.0);
        let crunch_attack = (t_crunch / 0.004).min(1.0);
        let bite2 = crunch_attack * (-t_crunch / 0.04).exp() * 0.5;
        let release = (1.0 - progress).powf(0.5);
        let env = attack * (bite1 + bite2) * release;

        // Cartoon bite timbre with odd harmonics and soft saturation
        let raw = phase.sin() + 0.3 * (2.0 * phase).sin() + 0.15 * (3.0 * phase).sin();
        let sat = raw.tanh() * env * 0.75;

        samples.push(sat * center_gain);
        samples.push(sat * center_gain);
    }
    samples
}

/// Generates a celebratory ascending fanfare arpeggio.
///
/// - 4-note ascending major arpeggio: C5 (523 Hz), E5 (659 Hz), G5 (784 Hz), C6 (1046 Hz).
/// - Duration: ~0.54 seconds with sustained final chime and vibrato shimmer.
pub fn generate_correct_fanfare(sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let note_durations = [0.08_f32, 0.08, 0.08, 0.30];
    let note_freqs = [523.25_f32, 659.25, 783.99, 1046.50];
    let total_duration: f32 = note_durations.iter().sum();
    let num_frames = (total_duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_frames * 2);

    let two_pi = 2.0 * std::f32::consts::PI;
    let dt = 1.0 / sample_rate as f32;
    let center_gain = std::f32::consts::FRAC_1_SQRT_2;

    let mut current_note_idx = 0;
    let mut note_start_time = 0.0_f32;
    let mut phase = 0.0_f32;

    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;

        if current_note_idx + 1 < note_durations.len() {
            let note_end_time = note_start_time + note_durations[current_note_idx];
            if t >= note_end_time {
                note_start_time = note_end_time;
                current_note_idx += 1;
            }
        }

        let base_freq = note_freqs[current_note_idx];
        let t_note = t - note_start_time;

        // Vibrato shimmer on the final ringing note
        let freq = if current_note_idx == 3 {
            base_freq * (1.0 + 0.008 * (two_pi * 6.0 * t_note).sin())
        } else {
            base_freq
        };

        phase += two_pi * freq * dt;
        if phase > two_pi {
            phase -= two_pi;
        }

        let attack = (t_note / 0.005).min(1.0);
        let decay_factor = if current_note_idx == 3 { 0.12 } else { 0.05 };
        let decay = (-t_note / decay_factor).exp();
        let env = attack * decay;

        let s1 = phase.sin() * 0.6;
        let s2 = (2.0 * phase).sin() * 0.25;
        let s3 = (3.0 * phase).sin() * 0.15;
        let mono = (s1 + s2 + s3) * env * 0.7;

        samples.push(mono * center_gain);
        samples.push(mono * center_gain);
    }
    samples
}

/// Generates a playful frequency-modulated gentle wobble sound effect.
///
/// - Carrier frequency: 340 Hz.
/// - LFO rate: 9 Hz vibrato with 60 Hz modulation depth.
/// - Duration: ~0.40 seconds with smooth trapezoidal fade in / fade out.
pub fn generate_gentle_wobble(sample_rate: u32) -> Vec<f32> {
    if sample_rate == 0 {
        return Vec::new();
    }
    let duration_secs = 0.40_f32;
    let num_frames = (duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_frames * 2);

    let two_pi = 2.0 * std::f32::consts::PI;
    let dt = 1.0 / sample_rate as f32;
    let center_gain = std::f32::consts::FRAC_1_SQRT_2;

    let carrier_freq = 340.0_f32;
    let mod_rate = 9.0_f32;
    let mod_depth = 60.0_f32;

    let mut phase = 0.0_f32;

    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;
        let progress = (t / duration_secs).min(1.0);

        let instantaneous_freq = carrier_freq + mod_depth * (two_pi * mod_rate * t).sin();
        phase += two_pi * instantaneous_freq * dt;
        if phase > two_pi {
            phase -= two_pi;
        }

        let attack = (t / 0.04).min(1.0);
        let release = ((1.0 - progress) / 0.06).min(1.0);
        let env = attack * release;

        let raw = phase.sin() * 0.75 + (2.0 * phase).sin() * 0.15;
        let mono = raw * env * 0.7;

        samples.push(mono * center_gain);
        samples.push(mono * center_gain);
    }
    samples
}
