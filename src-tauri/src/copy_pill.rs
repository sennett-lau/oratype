//! Plan `copy-when-no-field`: the Copy pill. When a Dictate or Translate result is ready and focus
//! is clearly not on a text field, the result is not pasted. It is kept here instead, the
//! clipboard is left alone, and the pill shows the start of the text with a Copy button. The pill
//! closes on Copy, on Escape, when its countdown runs out, or when a new run starts.
//!
//! Plan `clipboard-safety`: the same happens when another app is in front than when the
//! recording started, or when OraType may not type (no Accessibility permission). Those results
//! used to be copied to the clipboard on their own, over whatever the user had copied.

use crate::output::focus::FocusDecision;
use crate::output::InsertionStrategy;
use std::sync::{Arc, Mutex};

/// Event that shows the Copy pill (payload [`CopyOffer`]) or closes it (payload `null`).
pub const COPY_OFFER_EVENT: &str = "pipeline:copy_offer";

/// What the pill shows: the full result (it truncates the preview itself) and, for a
/// translation, the target language code for the language tag.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyOffer {
    pub text: String,
    pub target_lang: Option<String>,
}

/// A run that may end in the Copy pill (Dictate and Translate; Ask never does). Carries the
/// language tag the pill would show.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CopyPillRun {
    pub target_lang: Option<String>,
}

/// The result waiting in the pill, if the pill is up. Cloned handles share one slot.
#[derive(Debug, Clone, Default)]
pub struct CopyOfferSlot(Arc<Mutex<Option<String>>>);

impl CopyOfferSlot {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<String>> {
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }

    /// Keeps `text` for the pill, replacing an older offer.
    pub fn hold(&self, text: &str) {
        *self.lock() = Some(text.to_string());
    }

    /// True while the pill offers a result (Escape then closes the pill).
    pub fn is_up(&self) -> bool {
        self.lock().is_some()
    }

    /// Takes the result for Copy; the pill then shows "Copied" and closes by itself.
    pub fn take(&self) -> Option<String> {
        self.lock().take()
    }

    /// Drops the offer. Returns true when there was one.
    pub fn clear(&self) -> bool {
        self.lock().take().is_some()
    }
}

/// Paths that paste or type into the focused app. Copy-only output never pastes, so it never
/// needs the pill.
fn strategy_pastes(strategy: InsertionStrategy) -> bool {
    strategy != InsertionStrategy::ClipboardCopyOnly
}

/// Why a result goes to the Copy pill instead of being typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldReason {
    /// Focus is clearly not on a text field.
    NoTextField,
    /// Plan `clipboard-safety`: another app is in front than when the recording started.
    TargetChanged,
    /// Plan `clipboard-safety`: OraType may not type or paste (no Accessibility permission).
    CannotType,
}

/// Whether, and why, a result goes to the Copy pill instead of being typed. `strategy` is the
/// user's insertion setting; with copy-only set they asked for the clipboard, so nothing is
/// held. `focus` runs only when it matters (it asks Accessibility), and only a clear "not a text
/// field" holds the result there: unknown focus types as before.
pub fn copy_pill_hold(
    run: Option<&CopyPillRun>,
    strategy: InsertionStrategy,
    target_still_matches: bool,
    can_type: bool,
    focus: impl FnOnce() -> FocusDecision,
) -> Option<HoldReason> {
    if run.is_none() || !strategy_pastes(strategy) {
        return None;
    }
    if !target_still_matches {
        return Some(HoldReason::TargetChanged);
    }
    if !can_type {
        return Some(HoldReason::CannotType);
    }
    (focus() == FocusDecision::NotEditable).then_some(HoldReason::NoTextField)
}

/// Whether live streaming into the app may start. Streaming types while the AI is still
/// writing, so the focus is checked before it starts: with no text field, the run does not
/// stream and its final result goes through [`should_hold_for_copy_pill`] (checked again then,
/// because the user may click into a field meanwhile).
pub fn streaming_allowed(run: Option<&CopyPillRun>, focus: impl FnOnce() -> FocusDecision) -> bool {
    run.is_none() || focus() != FocusDecision::NotEditable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> CopyPillRun {
        CopyPillRun::default()
    }

    #[test]
    fn a_dictation_with_no_text_field_is_held_for_the_pill() {
        for strategy in [InsertionStrategy::Auto, InsertionStrategy::ClipboardPaste] {
            assert_eq!(
                copy_pill_hold(Some(&run()), strategy, true, true, || {
                    FocusDecision::NotEditable
                }),
                Some(HoldReason::NoTextField)
            );
        }
    }

    #[test]
    fn editable_or_unknown_focus_types_as_before() {
        for focus in [FocusDecision::Editable, FocusDecision::Unknown] {
            assert_eq!(
                copy_pill_hold(Some(&run()), InsertionStrategy::Auto, true, true, || focus),
                None
            );
        }
    }

    #[test]
    fn a_changed_app_or_no_permission_is_held_instead_of_copied() {
        let never = || -> FocusDecision { panic!("focus must not be checked") };
        assert_eq!(
            copy_pill_hold(Some(&run()), InsertionStrategy::Auto, false, true, never),
            Some(HoldReason::TargetChanged)
        );
        assert_eq!(
            copy_pill_hold(
                Some(&run()),
                InsertionStrategy::Keyboard,
                true,
                false,
                never
            ),
            Some(HoldReason::CannotType)
        );
    }

    #[test]
    fn ask_and_a_copy_only_setting_are_never_held() {
        let never = || -> FocusDecision { panic!("focus must not be checked") };
        assert_eq!(
            copy_pill_hold(None, InsertionStrategy::Auto, false, false, never),
            None
        );
        assert_eq!(
            copy_pill_hold(
                Some(&run()),
                InsertionStrategy::ClipboardCopyOnly,
                false,
                false,
                never
            ),
            None
        );
    }

    #[test]
    fn streaming_is_skipped_only_when_there_is_no_text_field() {
        assert!(!streaming_allowed(Some(&run()), || {
            FocusDecision::NotEditable
        }));
        assert!(streaming_allowed(Some(&run()), || FocusDecision::Unknown));
        assert!(streaming_allowed(Some(&run()), || FocusDecision::Editable));
        assert!(streaming_allowed(None, || panic!("Ask does not check")));
    }

    #[test]
    fn the_slot_keeps_one_offer_until_copied_or_closed() {
        let slot = CopyOfferSlot::default();
        assert!(!slot.is_up());
        slot.hold("first");
        slot.hold("second");
        let shared = slot.clone();
        assert!(shared.is_up());
        assert_eq!(shared.take().as_deref(), Some("second"));
        assert!(!slot.is_up());
        assert!(!slot.clear());
        slot.hold("third");
        assert!(slot.clear());
        assert_eq!(slot.take(), None);
    }

    #[test]
    fn the_offer_serializes_for_the_frontend() {
        let offer = CopyOffer {
            text: "Hi".to_string(),
            target_lang: Some("ja".to_string()),
        };
        assert_eq!(
            serde_json::to_value(offer).unwrap(),
            serde_json::json!({ "text": "Hi", "targetLang": "ja" })
        );
    }
}
