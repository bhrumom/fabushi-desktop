use crate::actor::{ActorId, Participant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConversationId(pub String);

impl ConversationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn is_valid(&self) -> bool {
        let value = self.0.trim();
        !value.is_empty() && value.len() <= 200
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConversationKind {
    Direct,
    Group,
    Channel,
    SavedMessages,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HistoryVisibility {
    NewMembersOnly,
    AllMembers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub muted_until_ms: Option<i64>,
    pub sound: Option<String>,
    pub show_preview: bool,
    pub notify_mentions: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            muted_until_ms: None,
            sound: None,
            show_preview: true,
            notify_mentions: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationPermissions {
    pub can_send_messages: bool,
    pub can_send_media: bool,
    pub can_send_polls: bool,
    pub can_add_members: bool,
    pub can_pin_messages: bool,
    pub can_manage_topics: bool,
    pub can_manage_calls: bool,
}

impl Default for ConversationPermissions {
    fn default() -> Self {
        Self {
            can_send_messages: true,
            can_send_media: true,
            can_send_polls: true,
            can_add_members: true,
            can_pin_messages: true,
            can_manage_topics: true,
            can_manage_calls: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub created_by: ActorId,
    pub closed: bool,
    pub hidden: bool,
    #[serde(default)]
    pub unread_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: ConversationId,
    pub kind: ConversationKind,
    pub title: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub participants: Vec<Participant>,
    pub owner_id: Option<ActorId>,
    pub last_message_id: Option<String>,
    pub last_read_message_id: Option<String>,
    pub unread_count: u32,
    pub mention_count: u32,
    pub pinned_message_ids: Vec<String>,
    pub notification_settings: NotificationSettings,
    pub permissions: ConversationPermissions,
    pub history_visibility: HistoryVisibility,
    pub topics: Vec<Topic>,
    pub folder_ids: Vec<String>,
    pub archived: bool,
    pub pinned: bool,
    pub marked_unread: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl Conversation {
    pub fn direct(
        id: impl Into<String>,
        title: impl Into<String>,
        participants: Vec<Participant>,
        now_ms: i64,
    ) -> Self {
        Self {
            id: ConversationId::new(id),
            kind: ConversationKind::Direct,
            title: title.into(),
            description: None,
            avatar_url: None,
            participants,
            owner_id: None,
            last_message_id: None,
            last_read_message_id: None,
            unread_count: 0,
            mention_count: 0,
            pinned_message_ids: Vec::new(),
            notification_settings: NotificationSettings::default(),
            permissions: ConversationPermissions::default(),
            history_visibility: HistoryVisibility::AllMembers,
            topics: Vec::new(),
            folder_ids: Vec::new(),
            archived: false,
            pinned: false,
            marked_unread: false,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDraft {
    pub conversation_id: ConversationId,
    pub actor_id: ActorId,
    pub text: String,
    pub reply_to_message_id: Option<String>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicDraft {
    pub conversation_id: ConversationId,
    pub topic_id: String,
    pub actor_id: ActorId,
    pub text: String,
    pub reply_to_message_id: Option<String>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationFolder {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub conversation_ids: Vec<ConversationId>,
    pub include_contacts: bool,
    pub include_bots: bool,
    pub include_groups: bool,
    pub include_channels: bool,
    pub exclude_muted: bool,
    pub exclude_read: bool,
    pub exclude_archived: bool,
}


/// Source-neutral identity for a selectable child destination inside the canonical
/// Conversation owner. A topic is keyed by its root Message identity, while a
/// saved sublist is keyed by the participant whose saved history it represents.
/// A nested Conversation covers community child histories without creating a
/// second Group/Channel/SavedMessages owner.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "kind")]
pub enum ConversationChildIdentity {
    Topic { root_message_id: String },
    SavedSublist { participant_id: ActorId },
    Conversation { conversation_id: ConversationId },
}

impl ConversationChildIdentity {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Topic { root_message_id } => {
                let value = root_message_id.trim();
                !value.is_empty() && value.len() <= 200
            }
            Self::SavedSublist { participant_id } => participant_id.is_valid(),
            Self::Conversation { conversation_id } => conversation_id.is_valid(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDestination {
    pub conversation_id: ConversationId,
    pub child: Option<ConversationChildIdentity>,
}

impl ConversationDestination {
    pub fn root(conversation_id: ConversationId) -> Self {
        Self {
            conversation_id,
            child: None,
        }
    }

    pub fn topic(
        conversation_id: ConversationId,
        root_message_id: impl Into<String>,
    ) -> Self {
        Self {
            conversation_id,
            child: Some(ConversationChildIdentity::Topic {
                root_message_id: root_message_id.into(),
            }),
        }
    }

    pub fn saved_sublist(
        conversation_id: ConversationId,
        participant_id: ActorId,
    ) -> Self {
        Self {
            conversation_id,
            child: Some(ConversationChildIdentity::SavedSublist { participant_id }),
        }
    }

    pub fn nested_conversation(
        conversation_id: ConversationId,
        child_conversation_id: ConversationId,
    ) -> Self {
        Self {
            conversation_id,
            child: Some(ConversationChildIdentity::Conversation {
                conversation_id: child_conversation_id,
            }),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.conversation_id.is_valid()
            && self.child.as_ref().map_or(true, ConversationChildIdentity::is_valid)
            && !matches!(
                &self.child,
                Some(ConversationChildIdentity::Conversation { conversation_id })
                    if conversation_id == &self.conversation_id
            )
    }

    /// Mirrors the upstream Thread distinction without importing its UI/runtime:
    /// topics and community histories can be marked read but not manually marked
    /// unread; self SavedSublist and monoforum-admin history rows cannot toggle
    /// unread at all.
    pub fn can_toggle_unread(
        &self,
        currently_unread: bool,
        context: ConversationChildUnreadContext,
    ) -> bool {
        if (matches!(&self.child, Some(ConversationChildIdentity::Topic { .. }))
            || context.parent_is_community)
            && !currently_unread
        {
            return false;
        }
        if matches!(
            &self.child,
            Some(ConversationChildIdentity::SavedSublist { .. })
        ) && context.parent_is_self
        {
            return false;
        }
        if self.child.is_none() && context.actor_is_monoforum_admin {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConversationChildUnreadContext {
    pub parent_is_self: bool,
    pub parent_is_community: bool,
    pub actor_is_monoforum_admin: bool,
}

/// Ephemeral picker selection over canonical Conversation identities. Destruction
/// is identity-specific: destroying a topic/sublist/nested child clears only an
/// exact matching selection and never collapses it silently to the parent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversationDestinationSelection {
    selected: Option<ConversationDestination>,
}

impl ConversationDestinationSelection {
    pub fn selected(&self) -> Option<&ConversationDestination> {
        self.selected.as_ref()
    }

    pub fn select(&mut self, destination: ConversationDestination) -> bool {
        if !destination.is_valid() {
            return false;
        }
        self.selected = Some(destination);
        true
    }

    pub fn deselect(&mut self) {
        self.selected = None;
    }

    pub fn clear_destroyed_child(
        &mut self,
        parent_conversation_id: &ConversationId,
        child: &ConversationChildIdentity,
    ) -> bool {
        let matches = self.selected.as_ref().is_some_and(|selected| {
            &selected.conversation_id == parent_conversation_id
                && selected.child.as_ref() == Some(child)
        });
        if matches {
            self.selected = None;
        }
        matches
    }
}

#[cfg(test)]
mod child_destination_tests {
    use super::*;

    #[test]
    fn topic_saved_sublist_and_nested_conversation_keep_distinct_identity() {
        let parent = ConversationId::new("conversation:parent");
        let topic = ConversationDestination::topic(parent.clone(), "topic:42");
        let saved = ConversationDestination::saved_sublist(
            parent.clone(),
            ActorId::new("human:42"),
        );
        let nested = ConversationDestination::nested_conversation(
            parent.clone(),
            ConversationId::new("conversation:child"),
        );

        assert!(topic.is_valid());
        assert!(saved.is_valid());
        assert!(nested.is_valid());
        assert_ne!(topic, saved);
        assert_ne!(saved, nested);
        assert_ne!(topic, nested);
        assert!(!ConversationDestination::nested_conversation(
            parent.clone(),
            parent,
        )
        .is_valid());
    }

    #[test]
    fn destroying_exact_child_clears_selection_without_parent_fallback() {
        let parent = ConversationId::new("conversation:parent");
        let topic_child = ConversationChildIdentity::Topic {
            root_message_id: "topic:42".into(),
        };
        let saved_child = ConversationChildIdentity::SavedSublist {
            participant_id: ActorId::new("human:42"),
        };
        let mut selection = ConversationDestinationSelection::default();
        assert!(selection.select(ConversationDestination {
            conversation_id: parent.clone(),
            child: Some(topic_child.clone()),
        }));

        assert!(!selection.clear_destroyed_child(&parent, &saved_child));
        assert!(selection.selected().is_some());
        assert!(selection.clear_destroyed_child(&parent, &topic_child));
        assert!(selection.selected().is_none());
    }

    #[test]
    fn unread_toggle_policy_preserves_topic_community_and_saved_sublist_rules() {
        let parent = ConversationId::new("conversation:parent");
        let topic = ConversationDestination::topic(parent.clone(), "topic:42");
        assert!(!topic.can_toggle_unread(
            false,
            ConversationChildUnreadContext::default(),
        ));
        assert!(topic.can_toggle_unread(
            true,
            ConversationChildUnreadContext::default(),
        ));

        let community = ConversationDestination::root(parent.clone());
        assert!(!community.can_toggle_unread(
            false,
            ConversationChildUnreadContext {
                parent_is_community: true,
                ..ConversationChildUnreadContext::default()
            },
        ));

        let saved = ConversationDestination::saved_sublist(
            parent.clone(),
            ActorId::new("human:42"),
        );
        assert!(!saved.can_toggle_unread(
            true,
            ConversationChildUnreadContext {
                parent_is_self: true,
                ..ConversationChildUnreadContext::default()
            },
        ));

        let monoforum_admin = ConversationDestination::root(parent);
        assert!(!monoforum_admin.can_toggle_unread(
            true,
            ConversationChildUnreadContext {
                actor_is_monoforum_admin: true,
                ..ConversationChildUnreadContext::default()
            },
        ));
    }
}
