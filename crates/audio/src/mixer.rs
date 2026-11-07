//! Asynchronous audio mixer worker and device management.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::synth;
use crate::SoundEffect;

/// Internal command sent across the unbounded channel to the audio worker thread.
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// Play the requested sound effect asynchronously.
    Play(SoundEffect),
    /// Terminate the audio worker thread event loop.
    Terminate,
}

/// Asynchronous audio mixer and procedural sound synthesizer.
///
/// Dispatches audio commands across an unbounded MPSC channel to a dedicated
/// background worker thread. Game loops calling `play` never block waiting for
/// audio device I/O or sample buffer generation.
///
/// In headless environments (such as CI without ALSA / PulseAudio), the mixer
/// gracefully enters a no-op dummy mode without panicking.
#[derive(Clone)]
pub struct AudioMixer {
    sender: Option<Sender<AudioCommand>>,
    is_headless: Arc<AtomicBool>,
    play_count: Arc<AtomicUsize>,
    worker_handle: Option<Arc<std::sync::Mutex<Option<JoinHandle<()>>>>>,
}

impl std::fmt::Debug for AudioMixer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioMixer")
            .field("is_headless", &self.is_headless)
            .field("play_count", &self.play_count)
            .finish()
    }
}

impl AudioMixer {
    /// Create a new `AudioMixer`, attempting to initialize the system audio output device.
    ///
    /// If device initialization fails (e.g. running in headless CI without ALSA/PulseAudio),
    /// a warning is logged and the mixer enters a no-op dummy mode without crashing.
    pub fn new() -> Self {
        if std::env::var("MOTIONARCADE_HEADLESS_AUDIO")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            info!("MOTIONARCADE_HEADLESS_AUDIO set; starting AudioMixer in headless dummy mode");
            return Self::dummy();
        }
        Self::init(false)
    }

    /// Create an `AudioMixer` explicitly in headless dummy mode.
    ///
    /// In dummy mode, commands are still sent through the background channel to verify
    /// channel message passing, but no physical audio hardware device is initialized.
    pub fn dummy() -> Self {
        Self::init(true)
    }

    /// Alias for `dummy()`.
    pub fn new_headless() -> Self {
        Self::dummy()
    }

    /// Enqueue a sound effect for non-blocking asynchronous playback.
    ///
    /// Game loops never wait for audio device buffer writes or sample synthesis.
    pub fn play(&self, effect: SoundEffect) {
        self.play_count.fetch_add(1, Ordering::Relaxed);
        if let Some(ref sender) = self.sender {
            if let Err(err) = sender.send(AudioCommand::Play(effect)) {
                warn!("Failed to send audio command to worker thread: {:?}", err);
            }
        }
    }

    /// Returns `true` if the mixer is running in headless dummy mode.
    pub fn is_headless(&self) -> bool {
        self.is_headless.load(Ordering::SeqCst)
    }

    /// Returns the total number of sound play requests issued to this mixer.
    pub fn play_count(&self) -> usize {
        self.play_count.load(Ordering::Relaxed)
    }

    /// Send a termination signal to the audio worker thread.
    pub fn shutdown(&self) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(AudioCommand::Terminate);
        }
    }

    /// Signals shutdown and joins the worker thread if available.
    pub fn wait_for_shutdown(&self) {
        self.shutdown();
        if let Some(ref handle_lock) = self.worker_handle {
            if let Ok(mut guard) = handle_lock.lock() {
                if let Some(handle) = guard.take() {
                    let _ = handle.join();
                }
            }
        }
    }

    fn init(force_headless: bool) -> Self {
        let (tx, rx) = mpsc::channel::<AudioCommand>();
        let (init_tx, init_rx) = mpsc::channel::<bool>();
        let is_headless = Arc::new(AtomicBool::new(force_headless));
        let is_headless_worker = Arc::clone(&is_headless);
        let play_count = Arc::new(AtomicUsize::new(0));

        let builder = thread::Builder::new().name("motionarcade-audio-worker".to_string());
        let spawn_result = builder.spawn(move || {
            Self::worker_loop(rx, init_tx, force_headless, is_headless_worker);
        });

        match spawn_result {
            Ok(handle) => {
                let init_ok = init_rx
                    .recv_timeout(Duration::from_millis(500))
                    .unwrap_or(false);
                if !init_ok {
                    is_headless.store(true, Ordering::SeqCst);
                }
                Self {
                    sender: Some(tx),
                    is_headless,
                    play_count,
                    worker_handle: Some(Arc::new(std::sync::Mutex::new(Some(handle)))),
                }
            }
            Err(err) => {
                warn!(
                    "Failed to spawn audio worker thread: {}. Operating in dummy mode.",
                    err
                );
                is_headless.store(true, Ordering::SeqCst);
                Self {
                    sender: None,
                    is_headless,
                    play_count,
                    worker_handle: None,
                }
            }
        }
    }

    fn worker_loop(
        rx: Receiver<AudioCommand>,
        init_tx: Sender<bool>,
        force_headless: bool,
        is_headless_flag: Arc<AtomicBool>,
    ) {
        let mut stream_holder = None;
        let mut handle_opt = None;

        if !force_headless {
            match rodio::OutputStream::try_default() {
                Ok((stream, handle)) => {
                    info!("Audio output device initialized successfully");
                    stream_holder = Some(stream);
                    handle_opt = Some(handle);
                    let _ = init_tx.send(true);
                }
                Err(err) => {
                    warn!(
                        "Audio device initialization failed: {}. Entering headless dummy mode.",
                        err
                    );
                    is_headless_flag.store(true, Ordering::SeqCst);
                    let _ = init_tx.send(false);
                }
            }
        } else {
            let _ = init_tx.send(false);
        }

        while let Ok(cmd) = rx.recv() {
            match cmd {
                AudioCommand::Play(effect) => {
                    if let Some(ref handle) = handle_opt {
                        let sample_rate = 44100;
                        let samples = synth::generate_samples(&effect, sample_rate);
                        let buffer = rodio::buffer::SamplesBuffer::new(2, sample_rate, samples);
                        if let Err(err) = handle.play_raw(buffer) {
                            warn!("Failed to play sound buffer on audio device: {:?}", err);
                        }
                    } else {
                        debug!("Dummy audio mode: skipping hardware playback of {:?}", effect);
                    }
                }
                AudioCommand::Terminate => {
                    debug!("Audio worker received shutdown signal");
                    break;
                }
            }
        }

        drop(handle_opt);
        drop(stream_holder);
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
