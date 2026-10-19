//! Plan `hands-free-mode` (listening.md, wake-check.md): turns the microphone stream into short
//! voiced segments for the wake check (and, during a request, for the "Thanks, Sam" check), and
//! decides when a hands-free request has ended.
//!
//! Both are plain state machines fed with samples, so tests drive them without a microphone.

use super::gate::{GateEvent, VoiceGate, FRAME_SAMPLES};
use super::ring::SampleRing;
use crate::stt::silence::WINDOW_MS;

const SAMPLES_PER_MS: u64 = 16;
/// Audio history kept in memory.
pub const RING_MS: u64 = 3_000;
/// A wake segment ends after this much quiet. Shorter than the request's end-of-speech wait:
/// "Hey Sam" is usually followed by a short pause.
pub const WAKE_HANGOVER_MS: u32 = 300;
/// Audio kept before the gate noticed speech (soft first consonants).
pub const PRE_ROLL_MS: u64 = 200;
/// A segment is checked once this long, even if the speaker goes on ("Hey Sam, what is ...").
pub const MAX_WAKE_SEGMENT_MS: u64 = 2_000;
/// Less voiced audio than this is a click or a bump, not a wake phrase (the voice check's
/// minimum).
pub const MIN_WAKE_VOICED_MS: u32 = crate::stt::silence::MIN_VOICED_MS;

/// During a request, the end check reads this much audio before each pause, across earlier
/// pauses: a quiet "Thanks, … Sam" reaches the gate as two stretches, and "…, thanks Sam" may
/// come in one breath with the sentence before it.
pub const END_CHECK_MS: u64 = 2_500;

/// A stretch of audio worth a wake check. Memory only; never logged or stored.
#[derive(Debug, Clone, PartialEq)]
pub struct WakeCandidate {
    pub samples: Vec<i16>,
    pub voiced_ms: u32,
    /// Absolute sample positions of the stretch (since the segmenter was made).
    pub start: u64,
    pub end: u64,
}

/// Splits the stream into wake candidates.
pub struct WakeSegmenter {
    ring: SampleRing,
    gate: VoiceGate,
    pending: Vec<i16>,
    /// Absolute start of the current segment, while in speech.
    segment_start: Option<u64>,
    /// The current segment was already checked at its maximum length.
    checked: bool,
    /// Candidates for the end check: one at each pause, the last [`END_CHECK_MS`] of audio
    /// before it, and no check in the middle of long speech.
    end_check: bool,
}

impl Default for WakeSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl WakeSegmenter {
    pub fn new() -> Self {
        Self {
            ring: SampleRing::new((RING_MS * SAMPLES_PER_MS) as usize),
            gate: VoiceGate::new(WAKE_HANGOVER_MS),
            pending: Vec::with_capacity(FRAME_SAMPLES),
            segment_start: None,
            checked: false,
            end_check: false,
        }
    }

    /// A segmenter for the end check during a request: a candidate at each pause, holding the
    /// last [`END_CHECK_MS`] of audio before it (also from before an earlier pause).
    pub fn for_end_check() -> Self {
        Self {
            end_check: true,
            ..Self::new()
        }
    }

    /// Forgets all audio (pause, resume, after a wake).
    pub fn reset(&mut self) {
        self.ring.clear();
        self.gate.reset();
        self.pending.clear();
        self.segment_start = None;
        self.checked = false;
    }

    /// Feeds samples; returns the candidates that completed.
    pub fn feed(&mut self, samples: &[i16]) -> Vec<WakeCandidate> {
        let mut out = Vec::new();
        for &sample in samples {
            self.pending.push(sample);
            if self.pending.len() < FRAME_SAMPLES {
                continue;
            }
            self.ring.push(&self.pending);
            let event = self.gate.push_frame(&self.pending);
            self.pending.clear();
            if let Some(candidate) = self.on_event(event) {
                out.push(candidate);
            }
        }
        out
    }

    fn on_event(&mut self, event: GateEvent) -> Option<WakeCandidate> {
        let now = self.ring.position();
        match event {
            GateEvent::SpeechStarted { frames_ago } => {
                let back = frames_ago as u64 * FRAME_SAMPLES as u64 + PRE_ROLL_MS * SAMPLES_PER_MS;
                self.segment_start = Some(now.saturating_sub(back));
                self.checked = false;
                None
            }
            GateEvent::SpeechEnded { voiced_ms } => {
                let start = self.segment_start.take()?;
                let already = std::mem::replace(&mut self.checked, false);
                if already || voiced_ms < MIN_WAKE_VOICED_MS {
                    return None;
                }
                // Drop most of the trailing quiet, keep 100 ms.
                let end = now
                    .saturating_sub(
                        u64::from(WAKE_HANGOVER_MS.saturating_sub(100)) * SAMPLES_PER_MS,
                    )
                    .max(start);
                let start = if self.end_check {
                    end.saturating_sub(END_CHECK_MS * SAMPLES_PER_MS)
                        .max(self.ring.oldest())
                } else {
                    start
                };
                Some(WakeCandidate {
                    samples: self.ring.range(start, end),
                    voiced_ms,
                    start,
                    end,
                })
            }
            GateEvent::None => {
                let start = self.segment_start?;
                if self.end_check
                    || self.checked
                    || now - start < MAX_WAKE_SEGMENT_MS * SAMPLES_PER_MS
                {
                    return None;
                }
                self.checked = true;
                let voiced_ms = self.gate.voiced_ms();
                (voiced_ms >= MIN_WAKE_VOICED_MS).then(|| WakeCandidate {
                    samples: self.ring.range(start, now),
                    voiced_ms,
                    start,
                    end: now,
                })
            }
        }
    }
}

/// Plan `hands-free-mode` (routing.md): how long a hands-free request may stay quiet before it
/// ends. Long enough for a thinking pause in dictation; "Thanks, Sam" ends it at once.
pub const END_OF_SPEECH_MS: u32 = 3_000;
/// Less voice than this is a breath, a click or a bump, not speech: it neither counts as the
/// request having started nor ends it (the voice check's minimum).
pub const MIN_REQUEST_VOICED_MS: u32 = crate::stt::silence::MIN_VOICED_MS;
/// A request with no speech at all ends after this long (the user said only "Hey Sam").
pub const NO_SPEECH_TIMEOUT_MS: u32 = 5_000;

/// Why a hands-free request stopped recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The speaker was quiet for [`END_OF_SPEECH_MS`] after speaking.
    EndOfSpeech,
    /// Nothing was said within [`NO_SPEECH_TIMEOUT_MS`].
    NoSpeech,
    /// The request reached its maximum length.
    MaxLength,
    /// The speaker said "Thanks, Sam".
    EndPhrase,
}

impl StopReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::EndOfSpeech => "end of speech",
            Self::NoSpeech => "no speech",
            Self::MaxLength => "max length",
            Self::EndPhrase => "end phrase",
        }
    }
}

/// Watches a hands-free request's audio and says when to stop it.
pub struct AutoStop {
    gate: VoiceGate,
    pending: Vec<i16>,
    elapsed_ms: u32,
    max_ms: u32,
    heard_speech: bool,
}

impl AutoStop {
    /// `max_ms` is the request's maximum length (the recording limit).
    pub fn new(max_ms: u32) -> Self {
        Self {
            gate: VoiceGate::new(END_OF_SPEECH_MS),
            pending: Vec::with_capacity(FRAME_SAMPLES),
            elapsed_ms: 0,
            max_ms: max_ms.max(1_000),
            heard_speech: false,
        }
    }

    /// Feeds samples; returns why to stop, once a reason applies.
    pub fn feed(&mut self, samples: &[i16]) -> Option<StopReason> {
        for &sample in samples {
            self.pending.push(sample);
            if self.pending.len() < FRAME_SAMPLES {
                continue;
            }
            let event = self.gate.push_frame(&self.pending);
            self.pending.clear();
            self.elapsed_ms += WINDOW_MS;
            match event {
                // A breath or a click while the speaker waits for the pill is not the request.
                GateEvent::SpeechEnded { voiced_ms } if voiced_ms >= MIN_REQUEST_VOICED_MS => {
                    return Some(StopReason::EndOfSpeech)
                }
                GateEvent::SpeechStarted { .. } | GateEvent::SpeechEnded { .. } => {}
                GateEvent::None => {}
            }
            if self.gate.voiced_ms() >= MIN_REQUEST_VOICED_MS {
                self.heard_speech = true;
            }
            if self.elapsed_ms >= self.max_ms {
                return Some(StopReason::MaxLength);
            }
            if !self.heard_speech && self.elapsed_ms >= NO_SPEECH_TIMEOUT_MS {
                return Some(StopReason::NoSpeech);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::gate::synth::{noise_frame, tone_frame};
    use super::*;

    struct Room {
        seed: u32,
        phase: f64,
    }

    impl Room {
        fn new() -> Self {
            Self {
                seed: 11,
                phase: 0.0,
            }
        }
        fn quiet(&mut self, ms: u32) -> Vec<i16> {
            (0..ms / WINDOW_MS)
                .flat_map(|_| noise_frame(-65.0, &mut self.seed))
                .collect()
        }
        fn voice(&mut self, ms: u32) -> Vec<i16> {
            (0..ms / WINDOW_MS)
                .flat_map(|_| tone_frame(-25.0, &mut self.phase))
                .collect()
        }
    }

    #[test]
    fn a_short_phrase_becomes_one_candidate_with_pre_roll() {
        let mut room = Room::new();
        let mut segmenter = WakeSegmenter::new();
        assert!(segmenter.feed(&room.quiet(1_000)).is_empty());
        assert!(segmenter.feed(&room.voice(700)).is_empty());
        let candidates = segmenter.feed(&room.quiet(500));
        assert_eq!(candidates.len(), 1);
        let candidate = &candidates[0];
        assert_eq!(candidate.voiced_ms, 700);
        // 700 ms of voice + 200 ms pre-roll + 100 ms of tail, give or take a frame.
        let ms = candidate.samples.len() as u64 / SAMPLES_PER_MS;
        assert!((960..=1_060).contains(&ms), "{ms}");
    }

    #[test]
    fn clicks_and_silence_give_no_candidates() {
        let mut room = Room::new();
        let mut segmenter = WakeSegmenter::new();
        let mut audio = room.quiet(1_000);
        audio.extend(room.voice(60));
        audio.extend(room.quiet(1_000));
        audio.extend(room.voice(120));
        audio.extend(room.quiet(1_000));
        assert!(segmenter.feed(&audio).is_empty());
    }

    #[test]
    fn long_speech_is_checked_once_at_two_seconds() {
        let mut room = Room::new();
        let mut segmenter = WakeSegmenter::new();
        segmenter.feed(&room.quiet(500));
        let candidates = segmenter.feed(&room.voice(5_000));
        assert_eq!(candidates.len(), 1);
        let ms = candidates[0].samples.len() as u64 / SAMPLES_PER_MS;
        assert!((2_000..=2_100).contains(&ms), "{ms}");
        // Its end gives no second check.
        assert!(segmenter.feed(&room.quiet(500)).is_empty());
        // The next phrase is checked again.
        segmenter.feed(&room.voice(600));
        assert_eq!(segmenter.feed(&room.quiet(500)).len(), 1);
    }

    #[test]
    fn reset_drops_a_half_heard_phrase() {
        let mut room = Room::new();
        let mut segmenter = WakeSegmenter::new();
        segmenter.feed(&room.quiet(500));
        segmenter.feed(&room.voice(400));
        segmenter.reset();
        assert!(segmenter.feed(&room.quiet(600)).is_empty());
    }

    #[test]
    fn auto_stop_waits_for_speech_then_stops_after_the_pause() {
        let mut room = Room::new();
        let mut stop = AutoStop::new(60_000);
        assert_eq!(stop.feed(&room.quiet(1_000)), None);
        assert_eq!(stop.feed(&room.voice(1_500)), None);
        // A 2.5 s thinking pause inside the request does not stop it.
        assert_eq!(stop.feed(&room.quiet(2_500)), None);
        assert_eq!(stop.feed(&room.voice(800)), None);
        assert_eq!(stop.feed(&room.quiet(2_960)), None);
        assert_eq!(stop.feed(&room.quiet(60)), Some(StopReason::EndOfSpeech));
    }

    #[test]
    fn a_breath_before_the_request_does_not_end_it() {
        // The speaker waits for the pill; a 100 ms breath and the quiet after it are not a
        // finished request (it ended one in real use, 2 s after the wake phrase).
        let mut room = Room::new();
        let mut stop = AutoStop::new(60_000);
        assert_eq!(stop.feed(&room.quiet(500)), None);
        assert_eq!(stop.feed(&room.voice(100)), None);
        assert_eq!(stop.feed(&room.quiet(3_200)), None);
        assert_eq!(stop.feed(&room.voice(1_200)), None);
        assert_eq!(stop.feed(&room.quiet(3_020)), Some(StopReason::EndOfSpeech));
    }

    #[test]
    fn the_end_check_gets_the_last_seconds_at_each_pause() {
        let mut room = Room::new();
        let mut segmenter = WakeSegmenter::for_end_check();
        segmenter.feed(&room.quiet(500));
        // Long speech is not checked in the middle.
        assert!(segmenter.feed(&room.voice(6_000)).is_empty());
        let candidates = segmenter.feed(&room.quiet(500));
        assert_eq!(candidates.len(), 1);
        let ms = candidates[0].samples.len() as u64 / SAMPLES_PER_MS;
        assert_eq!(ms, END_CHECK_MS);
        // A short phrase after a pause comes with what was said before it ("Thanks, … Sam").
        segmenter.feed(&room.voice(800));
        let candidates = segmenter.feed(&room.quiet(500));
        assert_eq!(candidates.len(), 1);
        let ms = candidates[0].samples.len() as u64 / SAMPLES_PER_MS;
        assert_eq!(ms, END_CHECK_MS);
        // Right after the wake phrase there is less audio than that.
        let mut segmenter = WakeSegmenter::for_end_check();
        segmenter.feed(&room.quiet(300));
        segmenter.feed(&room.voice(800));
        let ms = segmenter.feed(&room.quiet(500))[0].samples.len() as u64 / SAMPLES_PER_MS;
        assert!((1_200..=1_300).contains(&ms), "{ms}");
    }

    #[test]
    fn candidates_say_where_they_are() {
        let mut room = Room::new();
        let mut segmenter = WakeSegmenter::new();
        segmenter.feed(&room.quiet(1_000));
        segmenter.feed(&room.voice(400));
        let first = segmenter.feed(&room.quiet(500)).remove(0);
        segmenter.feed(&room.voice(400));
        let second = segmenter.feed(&room.quiet(500)).remove(0);
        assert_eq!(first.end - first.start, first.samples.len() as u64);
        // The gap between them (end of one to start of the next) is under a second.
        let gap_ms = (second.start - first.end) / SAMPLES_PER_MS;
        assert!(gap_ms < 1_000, "{gap_ms}");
    }

    #[test]
    fn auto_stop_gives_up_without_speech_and_at_the_max_length() {
        let mut room = Room::new();
        let mut stop = AutoStop::new(30_000);
        assert_eq!(stop.feed(&room.quiet(4_980)), None);
        assert_eq!(stop.feed(&room.quiet(40)), Some(StopReason::NoSpeech));

        let mut stop = AutoStop::new(3_000);
        assert_eq!(stop.feed(&room.voice(2_980)), None);
        assert_eq!(stop.feed(&room.voice(40)), Some(StopReason::MaxLength));
    }
}
