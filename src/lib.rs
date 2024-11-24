//! # libxm-rs
//! A binding of [libxm](https://github.com/Artefact2/libxm/) for Rust.
//!
//! A small XM (FastTracker II Extended Module) player library.
//! Designed for easy integration in demos and such, and provides timing
//! functions for easy sync against specific instruments, samples or channels.
//!
//! # Example
//! ```no_run
//! use libxm::XMContext;
//! use std::fs::File;
//! use std::io::Read;
//!
//! // Read the contents of the module into `data`
//! let mut data = Vec::new();
//! File::open("song.xm").unwrap().read_to_end(&mut data).unwrap();
//!
//! let mut xm = XMContext::new(&data, 48000).unwrap();
//! xm.set_max_loop_count(1);
//!
//! let mut buffer = [0.0; 4096];
//! while xm.loop_count() == 0 {
//!     xm.generate_samples(&mut buffer);
//!     // The buffer is filled with stereo PCM data. Use it for whatever you need...
//! }
//! // The song has looped once.
//! ```
//!
//! # Example
//! ```no_run
//! use libxm::XMContext;
//!
//! fn audio_callback(xm: &mut XMContext, buffer: &mut [f32]) {
//!     xm.generate_samples(buffer);
//! }
//! ```

pub mod ffi;
use std::mem;

/// Possible errors from the `XMContext::new` method.
#[derive(Copy, Clone, Debug)]
pub enum XMError {
    /// An unknown error reported by libxm.
    /// This enum exists in order to gracefully handle future errors from
    /// newer versions of libxm.
    Unknown(ffi::c_int),
    /// The module data is corrupted or invalid
    ModuleDataNotSane,
    /// There was an issue allocating additional memory
    MemoryAllocationFailed,
}

/// The return values from `XMContext::get_playing_speed()`.
#[derive(Copy, Clone)]
pub struct PlayingSpeed {
    /// Beats per minute
    pub bpm: u16,
    /// Ticks per line
    pub tempo: u16,
}

/// The return values from `XMContext::get_position()`.
#[derive(Copy, Clone)]
pub struct Position {
    /// Pattern index in the POT (pattern order table)
    pub pattern_index: u8,
    /// Pattern number
    pub pattern: u8,
    /// Row number
    pub row: u8,
    /// Total number of generated samples
    pub samples: u64,
}

/// The XM context.
pub struct XMContext {
    raw: *mut ffi::xm_context_t,
}

unsafe impl Send for XMContext {}
unsafe impl Sync for XMContext {}

impl XMContext {
    /// Creates an XM context.
    ///
    /// # Parameters
    /// * `mod_data` - The contents of the module.
    /// * `rate` - The play rate in Hz. Recommended value is 48000.
    pub fn new(mod_data: &[u8], rate: u32) -> Result<XMContext, XMError> {
        unsafe {
            let mut raw: *mut ffi::xm_context = std::ptr::null_mut();

            let mod_data_ptr = mem::transmute(mod_data.as_ptr());
            let mod_data_len = mod_data.len() as ffi::size_t;

            let result = ffi::xm_create_context_safe(&mut raw, mod_data_ptr, mod_data_len, rate);
            match result {
                0 => Ok(XMContext { raw: raw }),
                1 => Err(XMError::ModuleDataNotSane),
                2 => Err(XMError::MemoryAllocationFailed),
                _ => Err(XMError::Unknown(result)),
            }
        }
    }

    /// Plays the module and puts the sound samples in the specified output buffer.
    /// The output is in stereo.
    #[inline]
    pub fn generate_samples(&mut self, output: &mut [f32]) -> usize {
        unsafe {
            // Output buffer must have a multiple-of-two length.
            assert!(output.len() % 2 == 0);

            let output_len = (output.len() / 2) as ffi::size_t;
            let samples = ffi::xm_generate_samples(self.raw, output.as_mut_ptr(), output_len);

            (samples * 2) as usize
        }
    }

    /// Sets the maximum number of times a module can loop.
    ///
    /// After the specified number of loops, calls to `generate_samples()` will
    /// generate silence.
    #[inline]
    pub fn set_max_loop_count(&mut self, loopcnt: u8) {
        unsafe {
            ffi::xm_set_max_loop_count(self.raw, loopcnt);
        }
    }

    /// Gets the loop count of the currently playing module.
    ///
    /// This value is 0 when the module is still playing, 1 when the module has
    /// looped once, etc.
    #[inline]
    pub fn loop_count(&self) -> u8 {
        unsafe { ffi::xm_get_loop_count(self.raw) }
    }

    /// Gets the module name as a byte slice. The string encoding is unknown.
    ///
    /// Returns None if the XM_STRINGS build setting is false.
    #[inline]
    pub fn module_name(&self) -> Option<&[u8]> {
        // Is name always UTF-8? Another encoding?
        unsafe {
            let name = ffi::xm_get_module_name(self.raw);
            if name.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(name).to_bytes())
            }
        }
    }

    /// Gets the tracker name as a byte slice. The string encoding is unknown.
    ///
    /// Returns None if the XM_STRINGS build setting is false.
    #[inline]
    pub fn tracker_name(&self) -> Option<&[u8]> {
        // Is name always UTF-8? Another encoding?
        unsafe {
            let name = ffi::xm_get_tracker_name(self.raw);
            if name.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(name).to_bytes())
            }
        }
    }

    /// Gets the number of channels.
    #[inline]
    pub fn number_of_channels(&self) -> u16 {
        unsafe { ffi::xm_get_number_of_channels(self.raw) }
    }

    /// Gets the module length (in patterns).
    #[inline]
    pub fn module_length(&self) -> u16 {
        unsafe { ffi::xm_get_module_length(self.raw) }
    }

    /// Gets the number of patterns.
    #[inline]
    pub fn number_of_patterns(&self) -> u16 {
        unsafe { ffi::xm_get_number_of_patterns(self.raw) }
    }

    /// Gets the number of rows in a pattern.
    ///
    /// # Note
    /// Pattern numbers go from `0` to `get_number_of_patterns() - 1`
    #[inline]
    pub fn number_of_rows(&self, pattern: u16) -> u16 {
        assert!(pattern < self.number_of_patterns());

        unsafe { ffi::xm_get_number_of_rows(self.raw, pattern) }
    }

    /// Gets the number of instruments.
    #[inline]
    pub fn number_of_instruments(&self) -> u16 {
        unsafe { ffi::xm_get_number_of_instruments(self.raw) }
    }

    /// Gets the number of samples of an instrument.
    ///
    /// # Note
    /// Instrument numbers go from `1` to `get_number_of_instruments()`
    #[inline]
    pub fn number_of_samples(&self, instrument: u16) -> u16 {
        assert!(instrument >= 1);
        assert!(instrument <= self.number_of_instruments());

        unsafe { ffi::xm_get_number_of_samples(self.raw, instrument) }
    }

    /// Gets the current module speed.
    #[inline]
    pub fn playing_speed(&self) -> PlayingSpeed {
        let (mut bpm, mut tempo) = (0, 0);
        unsafe { ffi::xm_get_playing_speed(self.raw, &mut bpm, &mut tempo) };

        PlayingSpeed {
            bpm: bpm,
            tempo: tempo,
        }
    }

    /// Gets the current position in the module being played.
    #[inline]
    pub fn position(&self) -> Position {
        let (mut pattern_index, mut pattern, mut row) = (0, 0, 0);
        let mut samples = 0;
        unsafe {
            ffi::xm_get_position(
                self.raw,
                &mut pattern_index,
                &mut pattern,
                &mut row,
                &mut samples,
            )
        };

        Position {
            pattern_index: pattern_index,
            pattern: pattern,
            row: row,
            samples: samples,
        }
    }

    /// Gets the latest time (in number of generated samples) when a
    /// particular instrument was triggered in any channel.
    ///
    /// # Note
    /// Instrument numbers go from `1` to `get_number_of_instruments()`
    #[inline]
    pub fn latest_trigger_of_instrument(&self, instrument: u16) -> u64 {
        assert!(instrument >= 1);
        assert!(instrument <= self.number_of_instruments());

        unsafe { ffi::xm_get_latest_trigger_of_instrument(self.raw, instrument) }
    }

    /// Get the latest time (in number of generated samples) when a
    /// particular sample was triggered in any channel.
    ///
    /// # Note
    /// Instrument numbers go from `1` to `get_number_of_instruments()`
    ///
    /// Sample numbers go from `0` to `get_number_of_samples(instrument) - 1`
    #[inline]
    pub fn latest_trigger_of_sample(&self, instrument: u16, sample: u16) -> u64 {
        assert!(instrument >= 1);
        assert!(instrument <= self.number_of_instruments());
        assert!(sample < self.number_of_samples(instrument));

        unsafe { ffi::xm_get_latest_trigger_of_sample(self.raw, instrument, sample) }
    }

    /// Get the latest time (in number of generated samples) when any
    /// instrument was triggered in a given channel.
    ///
    /// # Note
    /// Channel numbers go from `1` to `get_number_of_channels()`
    #[inline]
    pub fn latest_trigger_of_channel(&self, channel: u16) -> u64 {
        assert!(channel >= 1);
        assert!(channel <= self.number_of_channels());

        unsafe { ffi::xm_get_latest_trigger_of_channel(self.raw, channel) }
    }

    /// Seek to a specific position in a module
    ///
    /// WARNING: WITH BIG LETTERS: seeking modules is broken by design,
    /// don't expect miracles
    #[inline]
    pub fn seek(&mut self, pot: u8, row: u8, tick: u16) {
        unsafe {
            ffi::xm_seek(self.raw, pot, row, tick);
        }
    }

    /// Mute or unmute a channel
    ///
    /// # Note
    /// Channel numbers go from `1` to `get_number_of_channels()`
    ///
    /// # Return
    /// Whether the channel was muted
    pub fn mute_channel(&mut self, channel: u16, mute: bool) -> bool {
        assert!(channel >= 1);
        assert!(channel <= self.number_of_channels());

        unsafe { ffi::xm_mute_channel(self.raw, channel, mute) }
    }

    /// Mute or unmute a instrument
    ///
    /// # Note
    /// Instrument numbers go from `1` to `get_number_of_channels()`
    ///
    /// # Return
    /// Whether the channel was muted
    pub fn mute_instrument(&mut self, instrument: u16, mute: bool) -> bool {
        assert!(instrument >= 1);
        assert!(instrument <= self.number_of_instruments());

        unsafe { ffi::xm_mute_instrument(self.raw, instrument, mute) }
    }
}

impl Drop for XMContext {
    fn drop(&mut self) {
        unsafe {
            ffi::xm_free_context(self.raw);
        }
    }
}
