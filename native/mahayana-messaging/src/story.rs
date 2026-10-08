use crate::actor::ActorId;
use crate::message::{FormattedText, MediaRef};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoryId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StoryPrivacyKind {
    Everyone,
    Contacts,
    CloseFriends,
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryPrivacy {
    pub kind: StoryPrivacyKind,
    pub included_actor_ids: BTreeSet<ActorId>,
    pub excluded_actor_ids: BTreeSet<ActorId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryView {
    pub actor_id: ActorId,
    pub viewed_at_ms: i64,
    pub reaction: Option<String>,
    pub forwarded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoryProgressState {
    pub index: usize,
    pub total: usize,
    pub progress: f64,
}

impl Default for StoryProgressState {
    fn default() -> Self {
        Self { index: 0, total: 1, progress: 0.0 }
    }
}

impl StoryProgressState {
    /// Projects the source-neutral Story slider state used by a capability surface.
    /// Every show call resets playback progress before exposing bounded index/total.
    pub fn show(&mut self, index: usize, total: usize) {
        self.progress = 0.0;
        self.total = total.max(1);
        self.index = index.min(self.total - 1);
    }

    /// Updates only the active Story playback projection with a finite bounded value.
    pub fn update_playback(&mut self, progress: f64) {
        self.progress = if progress.is_finite() { progress.clamp(0.0, 1.0) } else { 0.0 };
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Story {
    pub id: StoryId,
    pub owner_id: ActorId,
    pub media: MediaRef,
    pub caption: FormattedText,
    pub privacy: StoryPrivacy,
    pub created_at_ms: i64,
    pub expires_at_ms: i64,
    pub edited_at_ms: Option<i64>,
    pub pinned_to_profile: bool,
    pub protected_content: bool,
    pub allow_replies: bool,
    pub views: BTreeMap<ActorId, StoryView>,
}

impl Story {
    pub fn is_visible_to(
        &self,
        actor_id: &ActorId,
        is_contact: bool,
        is_close_friend: bool,
    ) -> bool {
        if &self.owner_id == actor_id {
            return true;
        }
        if self.privacy.excluded_actor_ids.contains(actor_id) {
            return false;
        }
        if self.privacy.included_actor_ids.contains(actor_id) {
            return true;
        }
        match self.privacy.kind {
            StoryPrivacyKind::Everyone => true,
            StoryPrivacyKind::Contacts => is_contact,
            StoryPrivacyKind::CloseFriends => is_close_friend,
            StoryPrivacyKind::Selected => false,
        }
    }

    pub fn record_view(&mut self, actor_id: ActorId, viewed_at_ms: i64) -> Result<(), StoryError> {
        if viewed_at_ms > self.expires_at_ms && !self.pinned_to_profile {
            return Err(StoryError::Expired(self.id.clone()));
        }
        self.views
            .entry(actor_id.clone())
            .and_modify(|view| view.viewed_at_ms = view.viewed_at_ms.min(viewed_at_ms))
            .or_insert(StoryView {
                actor_id,
                viewed_at_ms,
                reaction: None,
                forwarded: false,
            });
        Ok(())
    }

    pub fn react(
        &mut self,
        actor_id: &ActorId,
        reaction: Option<String>,
    ) -> Result<(), StoryError> {
        let view = self
            .views
            .get_mut(actor_id)
            .ok_or_else(|| StoryError::ViewerNotFound(actor_id.clone()))?;
        view.reaction = reaction;
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StoryError {
    #[error("story {0:?} has expired")]
    Expired(StoryId),
    #[error("story viewer {0:?} was not found")]
    ViewerNotFound(ActorId),
}


#[cfg(test)]
mod progress_tests {
    use super::StoryProgressState;

    #[test]
    fn story_progress_show_normalizes_bounds_and_resets_playback() {
        let mut state = StoryProgressState::default();
        state.update_playback(0.75);
        state.show(9, 3);
        assert_eq!(state.index, 2);
        assert_eq!(state.total, 3);
        assert_eq!(state.progress, 0.0);
        state.update_playback(0.5);
        state.show(0, 0);
        assert_eq!(state.index, 0);
        assert_eq!(state.total, 1);
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn story_progress_updates_only_with_bounded_finite_values() {
        let mut state = StoryProgressState::default();
        state.update_playback(-1.0);
        assert_eq!(state.progress, 0.0);
        state.update_playback(2.0);
        assert_eq!(state.progress, 1.0);
        state.update_playback(f64::NAN);
        assert_eq!(state.progress, 0.0);
        state.update_playback(0.375);
        assert_eq!(state.progress, 0.375);
    }

    #[test]
    fn repeated_show_resets_progress_even_when_selection_is_unchanged() {
        let mut state = StoryProgressState::default();
        state.show(1, 3);
        state.update_playback(0.8);
        state.show(1, 3);
        assert_eq!(state, StoryProgressState { index: 1, total: 3, progress: 0.0 });
    }
}
