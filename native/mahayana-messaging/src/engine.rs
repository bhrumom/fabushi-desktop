use crate::actor::{Actor, ActorId, ActorKind, Participant, ParticipantRole, Presence, PresenceStatus};
use crate::bot::{BotExecution, BotInvocation, BotProfile, BotRegistry};
use crate::community::{
    AdminRights, CommunityAuditAction, CommunityAuditEntry, CommunityError, CommunityMember,
    CommunityState, ForumTopicState, InviteLink, JoinRequest, MemberStatus,
};
use crate::connected_app::{
    ConnectedAppClaimDecision, ConnectedAppError, ConnectedAppRequest, ConnectedAppSession,
    ConnectedAppState,
};
use crate::conversation::{
    Conversation, ConversationChildIdentity, ConversationChildRuntimeState,
    ConversationChildUnreadThings, ConversationDestination, ConversationDraft, ConversationFolder, ConversationId,
    ConversationKind, ConversationMessagePosition, NotificationSettings, TopicDraft,
};
use crate::message::{
    ClientMessageId, DeliveryState, ForwardPrivacy, Message, MessageContent, MessageId,
    PendingPresenceSend, PresenceSendTrigger, ReactionSummary,
};
use crate::miniapp::{
    MiniAppGrant, MiniAppManifest, MiniAppPermission, MiniAppRequest, MiniAppResponse,
    MiniAppSession,
};
use crate::payment::{CustomerInfo, Entitlement, Invoice, Money, PaymentOrder, PaymentStatus};
use crate::story::{
    Story, StoryError, StoryId, StoryStealthState, STORY_STEALTH_ACTIVE_MS,
    STORY_STEALTH_COOLDOWN_MS, STORY_STEALTH_PRODUCT_ID, STORY_STEALTH_RETROACTIVE_MS,
};
use crate::wallet::{
    LedgerEntry, OnrampProviderInfo, OutboundTransferError, OutboundTransferRecord,
    OutboundTransferTerminal, WalletAccountId, WalletAddressDirectoryError, WalletError,
    WalletLedger, WalletLiveError, WalletLivePresence, WalletOnrampError, WalletRateError,
    WalletRuntimeState, WalletSponsoredFeeError, WalletSponsoredFeeInfo, WalletTransferIdentity,
    WalletTransferQuote, WalletTransferQuoteError,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum Command {
    UpsertActor {
        actor: Actor,
    },
    SetPresence {
        actor_id: ActorId,
        presence: Presence,
    },
    UpsertConversation {
        conversation: Conversation,
    },
    UpdateConversationInfo {
        conversation_id: ConversationId,
        title: String,
        description: Option<String>,
    },
    SetConversationParticipant {
        conversation_id: ConversationId,
        participant: Participant,
    },
    RemoveConversationParticipant {
        conversation_id: ConversationId,
        actor_id: ActorId,
    },
    ArchiveConversation {
        conversation_id: ConversationId,
        archived: bool,
    },
    PinConversation {
        conversation_id: ConversationId,
        pinned: bool,
    },
    SetMarkedUnread {
        conversation_id: ConversationId,
        actor_id: ActorId,
        marked_unread: bool,
    },
    SetDraft {
        draft: ConversationDraft,
    },
    SetConversationNotifications {
        conversation_id: ConversationId,
        settings: NotificationSettings,
    },
    UpsertFolder {
        folder: ConversationFolder,
    },
    DeleteFolder {
        folder_id: String,
    },
    QueueMessage {
        conversation_id: ConversationId,
        local_message_id: MessageId,
        client_message_id: ClientMessageId,
        sender_id: ActorId,
        content: MessageContent,
        reply_to_message_id: Option<MessageId>,
        thread_root_message_id: Option<MessageId>,
        created_at_ms: i64,
        scheduled_at_ms: Option<i64>,
        silent: bool,
        protected_content: bool,
    },
    QueuePresenceTriggeredSend {
        pending: PendingPresenceSend,
    },
    RemovePresenceTriggeredSend {
        client_message_id: ClientMessageId,
    },
    ForwardMessage {
        source_conversation_id: ConversationId,
        message_id: MessageId,
        destination_conversation_id: ConversationId,
        local_message_id: MessageId,
        client_message_id: ClientMessageId,
        sender_id: ActorId,
        thread_root_message_id: Option<MessageId>,
        created_at_ms: i64,
        scheduled_at_ms: Option<i64>,
        silent: bool,
        privacy: ForwardPrivacy,
    },
    AcknowledgeMessage {
        conversation_id: ConversationId,
        local_message_id: MessageId,
        server_message_id: MessageId,
        accepted_at_ms: i64,
    },
    SetDeliveryState {
        conversation_id: ConversationId,
        message_id: MessageId,
        state: DeliveryState,
    },
    EditMessage {
        conversation_id: ConversationId,
        message_id: MessageId,
        content: MessageContent,
        edited_at_ms: i64,
    },
    DeleteMessages {
        conversation_id: ConversationId,
        message_ids: Vec<MessageId>,
    },
    MarkRead {
        conversation_id: ConversationId,
        actor_id: ActorId,
        message_id: MessageId,
    },
    MarkTopicRead {
        conversation_id: ConversationId,
        topic_id: String,
        actor_id: ActorId,
        message_id: MessageId,
    },
    MarkConversationChildRead {
        destination: ConversationDestination,
        actor_id: ActorId,
        message_id: MessageId,
    },
    SetTopicDraft {
        draft: TopicDraft,
    },
    SetConversationChildDraft {
        destination: ConversationDestination,
        actor_id: ActorId,
        text: String,
        reply_to_message_id: Option<MessageId>,
        updated_at_ms: i64,
    },
    ReplaceConversationChildWindow {
        destination: ConversationDestination,
        actor_id: ActorId,
        message_ids: Vec<MessageId>,
        skipped_before: Option<u32>,
        skipped_after: Option<u32>,
        full_count: Option<u32>,
    },
    SetConversationChildPinned {
        destination: ConversationDestination,
        actor_id: ActorId,
        pinned: bool,
    },
    SetConversationChildActive {
        destination: ConversationDestination,
        actor_id: ActorId,
        active: bool,
    },
    SetConversationChildMarkedUnread {
        destination: ConversationDestination,
        actor_id: ActorId,
        marked_unread: bool,
    },
    /// Server-authoritative reconciliation for child-level unread signals and
    /// pending incoming notification ids. This intentionally has no
    /// ClientCommand counterpart: renderer/client code cannot mint unread truth.
    ReconcileConversationChildUnreadThings {
        destination: ConversationDestination,
        actor_id: ActorId,
        known: bool,
        mention_message_ids: Vec<MessageId>,
        reaction_message_ids: Vec<MessageId>,
        poll_vote_message_ids: Vec<MessageId>,
        pending_incoming_notification_message_ids: Vec<MessageId>,
    },
    /// Server/native authority for account-scoped SavedSublist parent access.
    /// No ClientCommand exposes this relation: a renderer cannot turn an
    /// arbitrary community into a SavedSublist parent.
    ReconcileSavedSublistParentAccess {
        conversation_id: ConversationId,
        actor_id: ActorId,
        allowed: bool,
    },
    /// Server/native authority for the exact messages owned by one SavedSublist.
    /// No ClientCommand exposes this relation: renderer-visible parent messages
    /// are never sufficient evidence of child membership.
    ReconcileSavedSublistMembership {
        destination: ConversationDestination,
        actor_id: ActorId,
        message_ids: Vec<MessageId>,
    },
    SetConversationChildNoPaidMessages {
        destination: ConversationDestination,
        actor_id: ActorId,
        no_paid_messages: bool,
    },
    DestroyConversationChild {
        destination: ConversationDestination,
        actor_id: ActorId,
    },
    SetReaction {
        conversation_id: ConversationId,
        message_id: MessageId,
        reaction: ReactionSummary,
    },
    PinMessage {
        conversation_id: ConversationId,
        message_id: MessageId,
        pinned: bool,
    },
    VotePoll {
        conversation_id: ConversationId,
        message_id: MessageId,
        actor_id: ActorId,
        option_ids: Vec<String>,
    },
    CreateInvoice {
        invoice: Invoice,
    },
    UpsertOrder {
        order: PaymentOrder,
    },
    CheckoutInvoice {
        invoice_id: String,
        order_id: String,
        buyer_id: ActorId,
        customer: Option<CustomerInfo>,
        created_at_ms: i64,
    },
    RefundOrder {
        order_id: String,
        seller_id: ActorId,
        request_id: String,
        refunded_at_ms: i64,
    },
    CreditWalletSettlement {
        request_id: String,
        owner_id: ActorId,
        amount: Money,
        reference: Option<String>,
        settled_at_ms: i64,
    },
    SetWalletFiatCurrency {
        currency: String,
    },
    ApplyWalletRateSnapshot {
        rates: BTreeMap<String, i64>,
        observed_at_ms: i64,
    },
    MarkWalletRateRefreshFailed {
        observed_at_ms: i64,
    },
    BeginWalletFunding {
        address: String,
        asset: String,
        base_currency: Option<String>,
    },
    ResolveWalletFundingProvider {
        request_id: u64,
        providers: Vec<OnrampProviderInfo>,
    },
    CompleteWalletFunding {
        request_id: u64,
        session_url: Option<String>,
    },
    CancelWalletFunding {
        request_id: u64,
    },
    ShowWalletPanel,
    MinimizeWalletPanel,
    CloseWalletPanel,
    SetWalletTransactionsVisible {
        visible: bool,
    },
    JournalOutboundTransfer {
        record: OutboundTransferRecord,
    },
    JournalQuotedOutboundTransfer {
        record: OutboundTransferRecord,
        quote: WalletTransferQuote,
        balance_nano: i64,
        observed_at_ms: i64,
    },
    MarkOutboundTransferHandoff {
        record_id: String,
        message_token: Vec<u8>,
    },
    RecordOutboundTransferLookup {
        record_id: String,
    },
    StopOutboundTransferLookup {
        record_id: String,
    },
    SettleOutboundTransfer {
        record_id: String,
        terminal: OutboundTransferTerminal,
        confirmed_hash: Option<Vec<u8>>,
    },
    ReconcileWalletUserAddress {
        actor_id: ActorId,
        serial: u64,
        address: Option<String>,
        public_key: Vec<u8>,
    },
    ReconcileWalletAddressOwner {
        address: String,
        actor_id: Option<ActorId>,
        public_key: Vec<u8>,
    },
    SetWalletAddressServiceUnavailable {
        unavailable: bool,
    },
    BeginWalletLiveGeneration,
    ReconcileWalletLivePresence {
        generation: u64,
        presence: WalletLivePresence,
        observed_at_ms: i64,
    },
    MarkWalletLiveStateFailed {
        generation: u64,
        observed_at_ms: i64,
    },
    MarkWalletStreamResynced {
        generation: u64,
        observed_at_ms: i64,
    },
    SpendWalletHistoryPageRequest {
        generation: u64,
    },
    RecordWalletHistoryProgress {
        generation: u64,
        visible_rows: usize,
    },
    RearmWalletHistoryWalk {
        generation: u64,
    },
    BeginWalletSponsoredFeeRequest {
        network_generation: u64,
        identity: WalletTransferIdentity,
        transfer_min_nano: i64,
        configured_min_nano: i64,
        observed_at_ms: i64,
        force: bool,
    },
    ApplyWalletSponsoredFeeInfo {
        serial: u64,
        network_generation: u64,
        identity: WalletTransferIdentity,
        info: WalletSponsoredFeeInfo,
        observed_at_ms: i64,
    },
    FailWalletSponsoredFeeRequest {
        serial: u64,
        network_generation: u64,
        identity: WalletTransferIdentity,
        observed_at_ms: i64,
    },
    ResetWalletSponsoredFeeGeneration {
        network_generation: u64,
    },
    UpsertConnectedAppSession {
        session: ConnectedAppSession,
    },
    QueueConnectedAppRequest {
        request: ConnectedAppRequest,
        observed_at_ms: i64,
    },
    ResolveConnectedAppRequest {
        session_id: u64,
        request_id: String,
        decision: ConnectedAppClaimDecision,
        operation_id: String,
        signed_payload: String,
        answer: Vec<u8>,
        observed_at_ms: i64,
    },
    ResolveConnectedAppWalletRequest {
        session_id: u64,
        request_id: String,
        decision: ConnectedAppClaimDecision,
        wallet_identity: WalletTransferIdentity,
        operation_id: String,
        signed_payload: String,
        answer: Vec<u8>,
        observed_at_ms: i64,
    },
    CloseConnectedAppSession {
        session_id: u64,
        closed_at_ms: i64,
    },
    PruneConnectedAppClaims {
        observed_at_ms: i64,
    },
    ReconcileEntitlement {
        entitlement: Entitlement,
    },
    ActivateStoryStealth {
        actor_id: ActorId,
        request_id: String,
        activated_at_ms: i64,
    },
    PublishStory {
        actor_id: ActorId,
        story: Story,
    },
    DeleteStory {
        actor_id: ActorId,
        story_id: StoryId,
    },
    ViewStory {
        actor_id: ActorId,
        story_id: StoryId,
        viewed_at_ms: i64,
    },
    ReactStory {
        actor_id: ActorId,
        story_id: StoryId,
        reaction: Option<String>,
        reacted_at_ms: i64,
    },
    UpdateCommunity {
        actor_id: ActorId,
        community: CommunityState,
    },
    SubscribeChannel {
        actor_id: ActorId,
        conversation_id: ConversationId,
        subscribed_at_ms: i64,
    },
    UnsubscribeChannel {
        actor_id: ActorId,
        conversation_id: ConversationId,
        unsubscribed_at_ms: i64,
    },
    SetCommunitySlowMode {
        actor_id: ActorId,
        conversation_id: ConversationId,
        seconds: Option<u32>,
        changed_at_ms: i64,
    },
    ModerateCommunityMember {
        actor_id: ActorId,
        conversation_id: ConversationId,
        member: CommunityMember,
        reason: Option<String>,
        decided_at_ms: i64,
    },
    SetCommunityMember {
        actor_id: ActorId,
        conversation_id: ConversationId,
        member: CommunityMember,
    },
    CreateInviteLink {
        actor_id: ActorId,
        invite: InviteLink,
    },
    RevokeInviteLink {
        actor_id: ActorId,
        conversation_id: ConversationId,
        invite_id: String,
        revoked_at_ms: i64,
    },
    RequestCommunityJoin {
        actor_id: ActorId,
        request: JoinRequest,
    },
    RespondCommunityJoin {
        actor_id: ActorId,
        conversation_id: ConversationId,
        requester_id: ActorId,
        approved: bool,
        decided_at_ms: i64,
    },
    UpsertForumTopic {
        actor_id: ActorId,
        topic: ForumTopicState,
    },
    DeleteForumTopic {
        actor_id: ActorId,
        conversation_id: ConversationId,
        topic_id: String,
    },
    RegisterBot {
        actor_id: ActorId,
        profile: BotProfile,
    },
    BeginBotInvocation {
        actor_id: ActorId,
        invocation: BotInvocation,
        created_at_ms: i64,
    },
    FinishBotExecution {
        actor_id: ActorId,
        execution_id: String,
        success: bool,
        finished_at_ms: i64,
        error: Option<String>,
    },
    InstallMiniApp {
        manifest: MiniAppManifest,
    },
    GrantMiniApp {
        grant: MiniAppGrant,
    },
    OpenMiniApp {
        session: MiniAppSession,
    },
    MiniAppCall {
        session_id: String,
        request_id: String,
        request: MiniAppRequest,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum Event {
    ActorUpserted {
        actor: Actor,
    },
    PresenceUpdated {
        actor_id: ActorId,
        presence: Presence,
    },
    ConversationUpserted {
        conversation: Conversation,
    },
    ConversationInfoUpdated {
        conversation_id: ConversationId,
        title: String,
        description: Option<String>,
    },
    ConversationParticipantUpserted {
        conversation_id: ConversationId,
        participant: Participant,
    },
    ConversationParticipantRemoved {
        conversation_id: ConversationId,
        actor_id: ActorId,
    },
    ConversationArchived {
        conversation_id: ConversationId,
        archived: bool,
    },
    ConversationPinned {
        conversation_id: ConversationId,
        pinned: bool,
    },
    ConversationMarkedUnread {
        conversation_id: ConversationId,
        actor_id: ActorId,
        marked_unread: bool,
    },
    DraftChanged {
        draft: ConversationDraft,
    },
    ConversationNotificationsUpdated {
        conversation_id: ConversationId,
        settings: NotificationSettings,
    },
    FolderUpserted {
        folder: ConversationFolder,
    },
    FolderDeleted {
        folder_id: String,
    },
    PresenceTriggeredSendQueued {
        pending: PendingPresenceSend,
    },
    PresenceTriggeredSendRemoved {
        client_message_id: ClientMessageId,
    },
    MessageQueued {
        message: Message,
    },
    MessageAcknowledged {
        conversation_id: ConversationId,
        local_message_id: MessageId,
        server_message_id: MessageId,
        accepted_at_ms: i64,
    },
    DeliveryStateUpdated {
        conversation_id: ConversationId,
        message_id: MessageId,
        state: DeliveryState,
    },
    MessageEdited {
        conversation_id: ConversationId,
        message_id: MessageId,
        content: MessageContent,
        edited_at_ms: i64,
    },
    MessagesDeleted {
        conversation_id: ConversationId,
        message_ids: Vec<MessageId>,
    },
    ConversationRead {
        conversation_id: ConversationId,
        actor_id: ActorId,
        message_id: MessageId,
    },
    TopicReadChanged {
        conversation_id: ConversationId,
        topic_id: String,
        actor_id: ActorId,
        message_id: MessageId,
    },
    ConversationChildReadChanged {
        destination: ConversationDestination,
        actor_id: ActorId,
        message_id: MessageId,
    },
    TopicDraftChanged {
        draft: TopicDraft,
    },
    ConversationChildDraftChanged {
        destination: ConversationDestination,
        actor_id: ActorId,
        text: String,
        reply_to_message_id: Option<String>,
        updated_at_ms: i64,
    },
    ConversationChildWindowReplaced {
        destination: ConversationDestination,
        actor_id: ActorId,
        message_ids: Vec<String>,
        skipped_before: Option<u32>,
        skipped_after: Option<u32>,
        full_count: Option<u32>,
    },
    ConversationChildPinnedChanged {
        destination: ConversationDestination,
        actor_id: ActorId,
        pinned: bool,
    },
    ConversationChildActiveChanged {
        destination: ConversationDestination,
        actor_id: ActorId,
        active: bool,
    },
    ConversationChildMarkedUnreadChanged {
        destination: ConversationDestination,
        actor_id: ActorId,
        marked_unread: bool,
    },
    ConversationChildUnreadThingsReconciled {
        destination: ConversationDestination,
        actor_id: ActorId,
        unread_things: ConversationChildUnreadThings,
        pending_incoming_notification_message_ids: Vec<String>,
    },
    SavedSublistParentAccessReconciled {
        conversation_id: ConversationId,
        actor_id: ActorId,
        allowed: bool,
    },
    SavedSublistMembershipReconciled {
        destination: ConversationDestination,
        actor_id: ActorId,
        message_ids: Vec<String>,
    },
    ConversationChildNoPaidMessagesChanged {
        destination: ConversationDestination,
        actor_id: ActorId,
        no_paid_messages: bool,
    },
    ConversationChildDestroyed {
        destination: ConversationDestination,
        actor_id: ActorId,
    },
    ReactionUpdated {
        conversation_id: ConversationId,
        message_id: MessageId,
        reaction: ReactionSummary,
    },
    MessagePinned {
        conversation_id: ConversationId,
        message_id: MessageId,
        pinned: bool,
    },
    PollVoteChanged {
        conversation_id: ConversationId,
        message_id: MessageId,
        actor_id: ActorId,
        option_ids: Vec<String>,
    },
    InvoiceCreated {
        invoice: Invoice,
    },
    OrderUpserted {
        order: PaymentOrder,
    },
    WalletChanged {
        wallet: WalletLedger,
        entry: LedgerEntry,
    },
    WalletRuntimeChanged {
        runtime: WalletRuntimeState,
    },
    ConnectedAppStateChanged {
        state: ConnectedAppState,
    },
    EntitlementReconciled {
        entitlement: Entitlement,
    },
    StoryStealthChanged {
        actor_id: ActorId,
        state: StoryStealthState,
    },
    StoryChanged {
        story: Story,
    },
    StoryDeleted {
        story_id: StoryId,
    },
    CommunityChanged {
        community: CommunityState,
    },
    BotRegistryChanged {
        registry: BotRegistry,
        profile: Option<BotProfile>,
        execution: Option<BotExecution>,
    },
    MiniAppInstalled {
        manifest: MiniAppManifest,
    },
    MiniAppGrantUpdated {
        grant: MiniAppGrant,
    },
    MiniAppOpened {
        session: MiniAppSession,
    },
    MiniAppResponded {
        session_id: String,
        request_id: String,
        response: MiniAppResponse,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MessagingState {
    pub actors: BTreeMap<ActorId, Actor>,
    pub conversations: BTreeMap<ConversationId, Conversation>,
    pub folders: BTreeMap<String, ConversationFolder>,
    pub messages: BTreeMap<ConversationId, BTreeMap<MessageId, Message>>,
    pub read_cursors: BTreeMap<ConversationId, BTreeMap<ActorId, MessageId>>,
    pub topic_read_cursors:
        BTreeMap<ConversationId, BTreeMap<ActorId, BTreeMap<String, MessageId>>>,
    pub poll_votes:
        BTreeMap<ConversationId, BTreeMap<MessageId, BTreeMap<ActorId, BTreeSet<String>>>>,
    pub marked_unread_by_actor: BTreeMap<ConversationId, BTreeSet<ActorId>>,
    pub drafts: BTreeMap<ConversationId, BTreeMap<ActorId, ConversationDraft>>,
    pub topic_drafts: BTreeMap<ConversationId, BTreeMap<ActorId, BTreeMap<String, TopicDraft>>>,
    /// Canonical source-neutral child lifecycle state. Topic-only legacy maps above
    /// remain protocol-compatibility projections until their load-time migration
    /// is completed; new saved-sublist/community child state belongs here.
    pub conversation_child_states: Vec<ConversationChildRuntimeState>,
    /// Server/native-authoritative account access to a Conversation that owns
    /// SavedSublist children. This is the source-neutral replacement for the
    /// upstream account-scoped parent/admin relation.
    pub saved_sublist_parent_access: BTreeMap<ConversationId, BTreeSet<ActorId>>,
    pub pending_presence_sends: BTreeMap<ClientMessageId, PendingPresenceSend>,
    pub invoices: BTreeMap<String, Invoice>,
    pub orders: BTreeMap<String, PaymentOrder>,
    pub wallet: WalletLedger,
    pub connected_apps: ConnectedAppState,
    pub entitlements: BTreeMap<String, Entitlement>,
    pub story_stealth: BTreeMap<ActorId, StoryStealthState>,
    pub stories: BTreeMap<StoryId, Story>,
    pub communities: BTreeMap<ConversationId, CommunityState>,
    pub bots: BotRegistry,
    pub mini_apps: BTreeMap<String, MiniAppManifest>,
    pub mini_app_grants: BTreeMap<(String, ActorId), MiniAppGrant>,
    pub mini_app_sessions: BTreeMap<String, MiniAppSession>,
}

impl MessagingState {
    fn child_state(
        &self,
        destination: &ConversationDestination,
        actor_id: &ActorId,
    ) -> Option<&ConversationChildRuntimeState> {
        self.conversation_child_states
            .iter()
            .find(|state| &state.destination == destination && &state.actor_id == actor_id)
    }

    fn child_state_mut(
        &mut self,
        destination: ConversationDestination,
        actor_id: ActorId,
    ) -> Option<&mut ConversationChildRuntimeState> {
        if let Some(index) = self.conversation_child_states.iter().position(|state| {
            state.destination == destination && state.actor_id == actor_id
        }) {
            return self.conversation_child_states.get_mut(index);
        }
        let state = ConversationChildRuntimeState::new(destination, actor_id)?;
        self.conversation_child_states.push(state);
        self.conversation_child_states.last_mut()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EngineError {
    #[error("actor identifier is invalid")]
    InvalidActor,
    #[error("actor {0:?} does not exist")]
    ActorNotFound(ActorId),
    #[error("conversation identifier is invalid")]
    InvalidConversation,
    #[error("conversation {0:?} does not exist")]
    ConversationNotFound(ConversationId),
    #[error("conversation participant data is invalid")]
    InvalidConversationParticipant,
    #[error("message {message_id:?} does not exist in conversation {conversation_id:?}")]
    MessageNotFound {
        conversation_id: ConversationId,
        message_id: MessageId,
    },
    #[error("presence-triggered send is invalid for this target or conversation")]
    InvalidPresenceTriggeredSend,
    #[error("presence-trigger target is already online")]
    PresenceTriggerAlreadySatisfied,
    #[error("client message id {0:?} conflicts with an existing presence-triggered send")]
    DuplicatePresenceTriggeredSend(ClientMessageId),
    #[error("message {message_id:?} already exists in conversation {conversation_id:?}")]
    DuplicateMessage {
        conversation_id: ConversationId,
        message_id: MessageId,
    },
    #[error("client message identifier is invalid")]
    InvalidClientMessageId,
    #[error("message id list must not be empty")]
    EmptyMessageList,
    #[error("poll vote is invalid")]
    InvalidPollVote,
    #[error("message is protected from forwarding")]
    ProtectedContent,
    #[error("actor {actor_id:?} is not a participant of conversation {conversation_id:?}")]
    SenderNotParticipant {
        conversation_id: ConversationId,
        actor_id: ActorId,
    },
    #[error("conversation {0:?} does not allow sending messages")]
    MessageSendPermissionDenied(ConversationId),
    #[error("conversation {0:?} does not allow sending media")]
    MediaSendPermissionDenied(ConversationId),
    #[error("conversation {0:?} does not allow sending polls")]
    PollSendPermissionDenied(ConversationId),
    #[error("secret conversations must contain exactly two participants")]
    InvalidSecretConversation,
    #[error("secret conversations only accept encrypted secret content or service messages")]
    SecretPlaintextRejected,
    #[error("secret content may only be sent inside a secret conversation")]
    SecretContentOutsideSecretConversation,
    #[error("secret message envelope identity does not match the conversation participants")]
    SecretEnvelopeMismatch,
    #[error("story is invalid")]
    InvalidStory,
    #[error("story {0:?} does not exist")]
    StoryNotFound(StoryId),
    #[error("only the story owner may modify story {0:?}")]
    StoryPermissionDenied(StoryId),
    #[error("Story stealth requires an active entitlement")]
    StoryStealthEntitlementRequired,
    #[error("Story stealth activation request is invalid")]
    InvalidStoryStealthRequest,
    #[error("Story stealth cooldown is active until {retry_at_ms}")]
    StoryStealthCooldown { retry_at_ms: i64 },
    #[error(transparent)]
    Story(#[from] StoryError),
    #[error("community {0:?} does not exist")]
    CommunityNotFound(ConversationId),
    #[error("community administration permission denied")]
    CommunityPermissionDenied,
    #[error("community {0:?} member is not allowed to send messages")]
    CommunitySendRestricted(ConversationId),
    #[error("community {0:?} member is not allowed to send media")]
    CommunityMediaRestricted(ConversationId),
    #[error("community {0:?} member is not allowed to send polls")]
    CommunityPollRestricted(ConversationId),
    #[error("actor {actor_id:?} does not have access to community {conversation_id:?}")]
    CommunityAccessDenied {
        conversation_id: ConversationId,
        actor_id: ActorId,
    },
    #[error("community {conversation_id:?} slow mode is active until {retry_at_ms}")]
    SlowModeActive {
        conversation_id: ConversationId,
        retry_at_ms: i64,
    },
    #[error("forum topic {topic_id:?} does not exist in community {conversation_id:?}")]
    ForumTopicNotFound {
        conversation_id: ConversationId,
        topic_id: String,
    },
    #[error("forum topic {topic_id:?} is closed in community {conversation_id:?}")]
    ForumTopicClosed {
        conversation_id: ConversationId,
        topic_id: String,
    },
    #[error("message does not belong to forum topic {topic_id:?}")]
    TopicMessageMismatch { topic_id: String },
    #[error("conversation child destination is invalid")]
    InvalidConversationChildDestination,
    #[error("message does not belong to the selected conversation child")]
    ConversationChildMessageMismatch,
    #[error(transparent)]
    Community(#[from] CommunityError),
    #[error("bot operation permission denied")]
    BotPermissionDenied,
    #[error("bot operation failed: {0}")]
    Bot(String),
    #[error("invoice is invalid")]
    InvalidInvoice,
    #[error("customer information does not satisfy invoice requirements")]
    InvalidCustomerInfo,
    #[error("invoice {0} does not exist")]
    InvoiceNotFound(String),
    #[error("payment order {0} does not exist")]
    OrderNotFound(String),
    #[error("payment invoice {0} has expired")]
    InvoiceExpired(String),
    #[error("payment order {0} conflicts with an existing order")]
    OrderConflict(String),
    #[error("only the invoice seller may refund order {0}")]
    RefundForbidden(String),
    #[error("payment order {0} is not refundable in its current state")]
    OrderNotRefundable(String),
    #[error(transparent)]
    Wallet(#[from] WalletError),
    #[error(transparent)]
    WalletRate(#[from] WalletRateError),
    #[error(transparent)]
    WalletOnramp(#[from] WalletOnrampError),
    #[error("wallet panel is not visible")]
    WalletPanelNotVisible,
    #[error(transparent)]
    OutboundTransfer(#[from] OutboundTransferError),
    #[error(transparent)]
    WalletAddressDirectory(#[from] WalletAddressDirectoryError),
    #[error(transparent)]
    WalletLive(#[from] WalletLiveError),
    #[error(transparent)]
    WalletSponsoredFee(#[from] WalletSponsoredFeeError),
    #[error(transparent)]
    WalletTransferQuote(#[from] WalletTransferQuoteError),
    #[error(transparent)]
    ConnectedApp(#[from] ConnectedAppError),
    #[error("Mini App {0} is not installed")]
    MiniAppNotFound(String),
    #[error("Mini App session {0} does not exist")]
    MiniAppSessionNotFound(String),
    #[error("Mini App permission {0:?} was not granted")]
    MiniAppPermissionDenied(MiniAppPermission),
    #[error("Mini App request cannot be completed by the pure domain engine")]
    MiniAppHostActionRequired,
}

const MAX_RECENT_OPEN_DESTINATIONS: usize = 32;

#[derive(Debug, Clone, Default)]
pub struct MessagingEngine {
    state: MessagingState,
    // Runtime-only canonical navigation projection. This intentionally does not
    // live in MessagingState: upstream recent-open Thread history is weak and
    // non-persistent, and restart must not resurrect stale child authority.
    recent_open_destinations: BTreeMap<ActorId, Vec<ConversationDestination>>,
}

#[derive(Debug, Clone, Copy)]
enum CommunityAdminAction {
    ChangeInfo,
    InviteMembers,
    BanMembers,
    ManageTopics,
    AddAdmins,
}

fn is_community_owner(community: &CommunityState, actor_id: &ActorId) -> bool {
    community
        .members
        .get(actor_id)
        .is_some_and(|member| matches!(member.status, MemberStatus::Owner))
}

fn require_community_admin(
    community: &CommunityState,
    actor_id: &ActorId,
    action: CommunityAdminAction,
) -> Result<(), EngineError> {
    let allowed = community.members.get(actor_id).is_some_and(|member| {
        if matches!(member.status, MemberStatus::Owner) {
            return true;
        }
        if !matches!(member.status, MemberStatus::Administrator) {
            return false;
        }
        match action {
            CommunityAdminAction::ChangeInfo => member.admin_rights.change_info,
            CommunityAdminAction::InviteMembers => member.admin_rights.invite_members,
            CommunityAdminAction::BanMembers => member.admin_rights.ban_members,
            CommunityAdminAction::ManageTopics => member.admin_rights.manage_topics,
            CommunityAdminAction::AddAdmins => member.admin_rights.add_admins,
        }
    });
    if allowed {
        Ok(())
    } else {
        Err(EngineError::CommunityPermissionDenied)
    }
}

pub(crate) fn topic_id_from_root(root: &MessageId) -> Option<&str> {
    root.0.strip_prefix("topic:").filter(|id| !id.is_empty())
}

fn community_has_access(community: &CommunityState, actor_id: &ActorId) -> bool {
    community
        .members
        .get(actor_id)
        .is_some_and(|member| !matches!(member.status, MemberStatus::Left | MemberStatus::Banned))
        || community.is_subscriber(actor_id)
}

fn destination_message_conversation_id(
    destination: &ConversationDestination,
) -> &ConversationId {
    match destination.child.as_ref() {
        Some(ConversationChildIdentity::Conversation { conversation_id }) => conversation_id,
        _ => &destination.conversation_id,
    }
}

fn append_community_audit(
    community: &mut CommunityState,
    actor_id: &ActorId,
    action: CommunityAuditAction,
    target_actor_id: Option<ActorId>,
    target_id: Option<String>,
    reason: Option<String>,
    created_at_ms: i64,
) {
    let id = format!(
        "audit:{}:{}:{}:{}",
        actor_id.0,
        created_at_ms,
        community.admin_log.len(),
        target_id.as_deref().unwrap_or("community")
    );
    community.append_audit(CommunityAuditEntry {
        id,
        actor_id: actor_id.clone(),
        action,
        target_actor_id,
        target_id,
        reason,
        created_at_ms,
    });
}

fn message_content_uses_media(content: &MessageContent) -> bool {
    matches!(
        content,
        MessageContent::Photo { .. }
            | MessageContent::Video { .. }
            | MessageContent::Animation { .. }
            | MessageContent::Audio { .. }
            | MessageContent::Voice { .. }
            | MessageContent::VideoNote { .. }
            | MessageContent::Document { .. }
            | MessageContent::Sticker { .. }
    )
}

fn wallet_account_id(actor_id: &ActorId) -> WalletAccountId {
    WalletAccountId(format!("wallet:{}", actor_id.0))
}

fn ensure_wallet_account(
    wallet: &mut WalletLedger,
    account_id: &WalletAccountId,
    owner_id: &ActorId,
    now_ms: i64,
) -> Result<(), WalletError> {
    if wallet.accounts.contains_key(account_id) {
        return Ok(());
    }
    wallet.create_account(account_id.clone(), owner_id.clone(), now_ms)
}

fn participant_for_community_member(member: &CommunityMember) -> Option<Participant> {
    let role = match member.status {
        MemberStatus::Owner => ParticipantRole::Owner,
        MemberStatus::Administrator => ParticipantRole::Admin,
        MemberStatus::Member => ParticipantRole::Member,
        MemberStatus::Restricted => ParticipantRole::Restricted,
        MemberStatus::Left | MemberStatus::Banned => return None,
    };
    Some(Participant {
        actor_id: member.actor_id.clone(),
        role,
        joined_at_ms: member.joined_at_ms,
        muted_until_ms: member.restrictions.until_ms,
    })
}

impl MessagingEngine {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_state(state: MessagingState) -> Self {
        Self {
            state,
            recent_open_destinations: BTreeMap::new(),
        }
    }
    pub fn state(&self) -> &MessagingState {
        &self.state
    }
    pub fn recent_open_destinations(
        &self,
        actor_id: &ActorId,
    ) -> &[ConversationDestination] {
        self.recent_open_destinations
            .get(actor_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub fn into_state(self) -> MessagingState {
        self.state
    }

    fn note_destination_opened(
        &mut self,
        actor_id: &ActorId,
        destination: &ConversationDestination,
    ) {
        let recent = self
            .recent_open_destinations
            .entry(actor_id.clone())
            .or_default();
        recent.retain(|item| item != destination);
        recent.insert(0, destination.clone());
        recent.truncate(MAX_RECENT_OPEN_DESTINATIONS);
    }

    fn remove_recent_destination(
        &mut self,
        actor_id: &ActorId,
        destination: &ConversationDestination,
    ) {
        let remove_actor = self
            .recent_open_destinations
            .get_mut(actor_id)
            .is_some_and(|recent| {
                recent.retain(|item| item != destination);
                recent.is_empty()
            });
        if remove_actor {
            self.recent_open_destinations.remove(actor_id);
        }
    }

    pub fn execute(&mut self, command: Command) -> Result<Vec<Event>, EngineError> {
        let events = self.decide(command)?;
        for event in events.iter().cloned() {
            self.apply(event);
        }
        Ok(events)
    }

    pub fn decide(&self, command: Command) -> Result<Vec<Event>, EngineError> {
        match command {
            Command::UpsertActor { actor } => {
                if !actor.id.is_valid() || actor.display_name.trim().is_empty() {
                    return Err(EngineError::InvalidActor);
                }
                Ok(vec![Event::ActorUpserted { actor }])
            }
            Command::SetPresence { actor_id, presence } => {
                self.require_actor(&actor_id)?;
                Ok(vec![Event::PresenceUpdated { actor_id, presence }])
            }
            Command::UpsertConversation { conversation } => {
                if !conversation.id.is_valid() || conversation.title.trim().is_empty() {
                    return Err(EngineError::InvalidConversation);
                }
                if let Some(existing) = self.state.conversations.get(&conversation.id) {
                    if matches!(
                        existing.kind,
                        ConversationKind::Group | ConversationKind::Channel
                    ) && (conversation.owner_id != existing.owner_id
                        || conversation.participants != existing.participants
                        || conversation.topics != existing.topics)
                    {
                        return Err(EngineError::InvalidConversationParticipant);
                    }
                }
                for participant in &conversation.participants {
                    self.require_actor(&participant.actor_id)?;
                }
                if matches!(
                    conversation.kind,
                    crate::conversation::ConversationKind::Secret
                ) && conversation.participants.len() != 2
                {
                    return Err(EngineError::InvalidSecretConversation);
                }
                Ok(vec![Event::ConversationUpserted { conversation }])
            }
            Command::UpdateConversationInfo {
                conversation_id,
                title,
                description,
            } => {
                self.require_conversation(&conversation_id)?;
                if title.trim().is_empty() || title.trim().len() > 200 {
                    return Err(EngineError::InvalidConversationParticipant);
                }
                Ok(vec![Event::ConversationInfoUpdated {
                    conversation_id,
                    title: title.trim().to_string(),
                    description: description
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty()),
                }])
            }
            Command::SetConversationParticipant {
                conversation_id,
                participant,
            } => {
                let conversation = self.require_conversation(&conversation_id)?;
                self.require_actor(&participant.actor_id)?;
                if conversation.owner_id.as_ref() == Some(&participant.actor_id)
                    && !matches!(participant.role, ParticipantRole::Owner)
                {
                    return Err(EngineError::InvalidConversationParticipant);
                }
                if matches!(
                    conversation.kind,
                    ConversationKind::Group | ConversationKind::Channel
                ) {
                    let mut community = self
                        .state
                        .communities
                        .get(&conversation_id)
                        .cloned()
                        .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                    let status = match participant.role {
                        ParticipantRole::Owner => MemberStatus::Owner,
                        ParticipantRole::Admin => MemberStatus::Administrator,
                        ParticipantRole::Member => MemberStatus::Member,
                        ParticipantRole::Restricted => MemberStatus::Restricted,
                    };
                    let mut member = community
                        .members
                        .get(&participant.actor_id)
                        .cloned()
                        .unwrap_or_else(|| CommunityMember {
                            actor_id: participant.actor_id.clone(),
                            status: MemberStatus::Member,
                            admin_title: None,
                            admin_rights: AdminRights::default(),
                            restrictions: Default::default(),
                            joined_at_ms: participant.joined_at_ms,
                            invited_by: None,
                        });
                    member.status = status;
                    member.joined_at_ms = participant.joined_at_ms;
                    community
                        .members
                        .insert(participant.actor_id.clone(), member);
                    Ok(vec![
                        Event::CommunityChanged { community },
                        Event::ConversationParticipantUpserted {
                            conversation_id,
                            participant,
                        },
                    ])
                } else {
                    Ok(vec![Event::ConversationParticipantUpserted {
                        conversation_id,
                        participant,
                    }])
                }
            }
            Command::RemoveConversationParticipant {
                conversation_id,
                actor_id,
            } => {
                let conversation = self.require_conversation(&conversation_id)?;
                if conversation.owner_id.as_ref() == Some(&actor_id) {
                    return Err(EngineError::InvalidConversationParticipant);
                }
                if matches!(
                    conversation.kind,
                    ConversationKind::Group | ConversationKind::Channel
                ) {
                    let mut community = self
                        .state
                        .communities
                        .get(&conversation_id)
                        .cloned()
                        .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                    if let Some(member) = community.members.get_mut(&actor_id) {
                        member.status = MemberStatus::Left;
                    }
                    community.subscribers.remove(&actor_id);
                    Ok(vec![
                        Event::CommunityChanged { community },
                        Event::ConversationParticipantRemoved {
                            conversation_id,
                            actor_id,
                        },
                    ])
                } else {
                    Ok(vec![Event::ConversationParticipantRemoved {
                        conversation_id,
                        actor_id,
                    }])
                }
            }
            Command::ArchiveConversation {
                conversation_id,
                archived,
            } => {
                self.require_conversation(&conversation_id)?;
                Ok(vec![Event::ConversationArchived {
                    conversation_id,
                    archived,
                }])
            }
            Command::PinConversation {
                conversation_id,
                pinned,
            } => {
                self.require_conversation(&conversation_id)?;
                Ok(vec![Event::ConversationPinned {
                    conversation_id,
                    pinned,
                }])
            }
            Command::SetMarkedUnread {
                conversation_id,
                actor_id,
                marked_unread,
            } => {
                self.require_conversation(&conversation_id)?;
                self.require_actor(&actor_id)?;
                Ok(vec![Event::ConversationMarkedUnread {
                    conversation_id,
                    actor_id,
                    marked_unread,
                }])
            }
            Command::SetDraft { draft } => {
                self.require_conversation(&draft.conversation_id)?;
                self.require_actor(&draft.actor_id)?;
                Ok(vec![Event::DraftChanged { draft }])
            }
            Command::SetConversationNotifications {
                conversation_id,
                settings,
            } => {
                self.require_conversation(&conversation_id)?;
                Ok(vec![Event::ConversationNotificationsUpdated {
                    conversation_id,
                    settings,
                }])
            }
            Command::UpsertFolder { folder } => Ok(vec![Event::FolderUpserted { folder }]),
            Command::DeleteFolder { folder_id } => Ok(vec![Event::FolderDeleted { folder_id }]),
            Command::QueuePresenceTriggeredSend { pending } => {
                self.require_actor(&pending.sender_id)?;
                let target_actor_id = match &pending.trigger {
                    PresenceSendTrigger::WhenParticipantOnline { actor_id } => actor_id,
                };
                let target = self.require_actor(target_actor_id)?;
                if target.kind != ActorKind::Human || &pending.sender_id == target_actor_id {
                    return Err(EngineError::InvalidPresenceTriggeredSend);
                }
                if target.presence.status == PresenceStatus::Online {
                    return Err(EngineError::PresenceTriggerAlreadySatisfied);
                }
                let conversation = self.require_conversation(&pending.conversation_id)?;
                if !matches!(conversation.kind, ConversationKind::Direct) {
                    return Err(EngineError::InvalidPresenceTriggeredSend);
                }
                let target_is_participant = conversation
                    .participants
                    .iter()
                    .any(|participant| &participant.actor_id == target_actor_id)
                    || conversation.owner_id.as_ref() == Some(target_actor_id);
                if !target_is_participant {
                    return Err(EngineError::InvalidPresenceTriggeredSend);
                }
                if let Some(existing) = self
                    .state
                    .pending_presence_sends
                    .get(&pending.client_message_id)
                {
                    if existing == &pending {
                        return Ok(Vec::new());
                    }
                    return Err(EngineError::DuplicatePresenceTriggeredSend(
                        pending.client_message_id.clone(),
                    ));
                }
                // Reuse the canonical send decision for all current conversation,
                // media/poll, membership, channel/community, topic and slow-mode gates.
                let validation = self.decide(Command::QueueMessage {
                    conversation_id: pending.conversation_id.clone(),
                    local_message_id: pending.local_message_id.clone(),
                    client_message_id: pending.client_message_id.clone(),
                    sender_id: pending.sender_id.clone(),
                    content: pending.content.clone(),
                    reply_to_message_id: pending.reply_to_message_id.clone(),
                    thread_root_message_id: pending.thread_root_message_id.clone(),
                    created_at_ms: pending.created_at_ms,
                    scheduled_at_ms: None,
                    silent: pending.silent,
                    protected_content: pending.protected_content,
                })?;
                if !matches!(validation.as_slice(), [Event::MessageQueued { .. }]) {
                    return Err(EngineError::InvalidPresenceTriggeredSend);
                }
                Ok(vec![Event::PresenceTriggeredSendQueued { pending }])
            }
            Command::RemovePresenceTriggeredSend { client_message_id } => {
                if self
                    .state
                    .pending_presence_sends
                    .contains_key(&client_message_id)
                {
                    Ok(vec![Event::PresenceTriggeredSendRemoved { client_message_id }])
                } else {
                    Ok(Vec::new())
                }
            }
            Command::QueueMessage {
                conversation_id,
                local_message_id,
                client_message_id,
                sender_id,
                content,
                reply_to_message_id,
                thread_root_message_id,
                created_at_ms,
                scheduled_at_ms,
                silent,
                protected_content,
            } => {
                let conversation = self.require_conversation(&conversation_id)?;
                self.require_actor(&sender_id)?;
                let sender_is_participant = conversation
                    .participants
                    .iter()
                    .any(|participant| participant.actor_id == sender_id)
                    || conversation.owner_id.as_ref() == Some(&sender_id);
                if !sender_is_participant {
                    return Err(EngineError::SenderNotParticipant {
                        conversation_id: conversation_id.clone(),
                        actor_id: sender_id.clone(),
                    });
                }
                if !conversation.permissions.can_send_messages {
                    return Err(EngineError::MessageSendPermissionDenied(
                        conversation_id.clone(),
                    ));
                }
                if message_content_uses_media(&content) && !conversation.permissions.can_send_media
                {
                    return Err(EngineError::MediaSendPermissionDenied(
                        conversation_id.clone(),
                    ));
                }
                if matches!(&content, MessageContent::Poll { .. })
                    && !conversation.permissions.can_send_polls
                {
                    return Err(EngineError::PollSendPermissionDenied(
                        conversation_id.clone(),
                    ));
                }
                if let Some(member) = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .and_then(|community| community.members.get(&sender_id))
                {
                    if matches!(member.status, MemberStatus::Left | MemberStatus::Banned)
                        || (matches!(member.status, MemberStatus::Restricted)
                            && member.restrictions.send_messages)
                    {
                        return Err(EngineError::CommunitySendRestricted(
                            conversation_id.clone(),
                        ));
                    }
                    if message_content_uses_media(&content)
                        && matches!(member.status, MemberStatus::Restricted)
                        && member.restrictions.send_media
                    {
                        return Err(EngineError::CommunityMediaRestricted(
                            conversation_id.clone(),
                        ));
                    }
                    if matches!(&content, MessageContent::Poll { .. })
                        && matches!(member.status, MemberStatus::Restricted)
                        && member.restrictions.send_polls
                    {
                        return Err(EngineError::CommunityPollRestricted(
                            conversation_id.clone(),
                        ));
                    }
                }
                if matches!(conversation.kind, ConversationKind::Channel) {
                    let can_post =
                        conversation.owner_id.as_ref() == Some(&sender_id)
                            || conversation.participants.iter().any(|participant| {
                                participant.actor_id == sender_id
                                    && matches!(
                                        participant.role,
                                        ParticipantRole::Owner | ParticipantRole::Admin
                                    )
                            })
                            || self.state.communities.get(&conversation_id).is_some_and(
                                |community| {
                                    community.members.get(&sender_id).is_some_and(|member| {
                                        matches!(member.status, MemberStatus::Administrator)
                                            && member.admin_rights.post_messages
                                    })
                                },
                            );
                    if !can_post {
                        return Err(EngineError::CommunitySendRestricted(
                            conversation_id.clone(),
                        ));
                    }
                }
                if let Some(community) = self.state.communities.get(&conversation_id) {
                    if let Some(thread_root) = &thread_root_message_id {
                        if let Some(topic_id) = topic_id_from_root(thread_root) {
                            let topic = community.topics.get(topic_id).ok_or_else(|| {
                                EngineError::ForumTopicNotFound {
                                    conversation_id: conversation_id.clone(),
                                    topic_id: topic_id.to_string(),
                                }
                            })?;
                            if topic.closed || topic.hidden {
                                return Err(EngineError::ForumTopicClosed {
                                    conversation_id: conversation_id.clone(),
                                    topic_id: topic_id.to_string(),
                                });
                            }
                        }
                    }
                    if let Some(seconds) = community.slow_mode_seconds {
                        let bypass = community.can_moderate(&sender_id);
                        if !bypass {
                            let latest_sender_message = self
                                .state
                                .messages
                                .get(&conversation_id)
                                .into_iter()
                                .flat_map(|messages| messages.values())
                                .filter(|message| {
                                    message.sender_id == sender_id
                                        && !message.deleted
                                        && message
                                            .scheduled_at_ms
                                            .is_none_or(|scheduled| scheduled <= created_at_ms)
                                })
                                .max_by_key(|message| (message.created_at_ms, message.id.clone()));
                            if let Some(message) = latest_sender_message {
                                let retry_at = message
                                    .created_at_ms
                                    .saturating_add(i64::from(seconds).saturating_mul(1_000));
                                if created_at_ms < retry_at {
                                    return Err(EngineError::SlowModeActive {
                                        conversation_id: conversation_id.clone(),
                                        retry_at_ms: retry_at,
                                    });
                                }
                            }
                        }
                    }
                }
                let is_secret_conversation = matches!(
                    conversation.kind,
                    crate::conversation::ConversationKind::Secret
                );
                let is_secret_content = matches!(&content, MessageContent::Secret { .. });
                let is_service_content = matches!(&content, MessageContent::Service { .. });
                if is_secret_conversation && !is_secret_content && !is_service_content {
                    return Err(EngineError::SecretPlaintextRejected);
                }
                if !is_secret_conversation && is_secret_content {
                    return Err(EngineError::SecretContentOutsideSecretConversation);
                }
                if let MessageContent::Secret { envelope } = &content {
                    let participant_ids = conversation
                        .participants
                        .iter()
                        .map(|participant| &participant.actor_id)
                        .collect::<Vec<_>>();
                    if envelope.conversation_id != conversation_id
                        || envelope.sender_id != sender_id
                        || !participant_ids.contains(&&envelope.sender_id)
                        || !participant_ids.contains(&&envelope.recipient_id)
                        || envelope.sender_id == envelope.recipient_id
                    {
                        return Err(EngineError::SecretEnvelopeMismatch);
                    }
                }
                if client_message_id.0.trim().is_empty() || client_message_id.0.len() > 200 {
                    return Err(EngineError::InvalidClientMessageId);
                }
                if self
                    .state
                    .messages
                    .get(&conversation_id)
                    .is_some_and(|messages| messages.contains_key(&local_message_id))
                {
                    return Err(EngineError::DuplicateMessage {
                        conversation_id,
                        message_id: local_message_id,
                    });
                }
                let message = Message {
                    id: local_message_id,
                    conversation_id,
                    sender_id,
                    content,
                    reply_to_message_id,
                    thread_root_message_id,
                    forward_origin: None,
                    reply_markup: None,
                    reactions: Vec::new(),
                    delivery_state: DeliveryState::Pending { client_message_id },
                    created_at_ms,
                    edited_at_ms: None,
                    scheduled_at_ms,
                    silent,
                    protected_content: protected_content || is_secret_content,
                    pinned: false,
                    deleted: false,
                };
                Ok(vec![Event::MessageQueued { message }])
            }
            Command::ForwardMessage {
                source_conversation_id,
                message_id,
                destination_conversation_id,
                local_message_id,
                client_message_id,
                sender_id,
                thread_root_message_id,
                created_at_ms,
                scheduled_at_ms,
                silent,
                privacy,
            } => {
                let conversation = self.require_conversation(&destination_conversation_id)?;
                self.require_actor(&sender_id)?;
                let original = self
                    .require_message(&source_conversation_id, &message_id)?
                    .clone();
                if original.deleted {
                    return Err(EngineError::MessageNotFound {
                        conversation_id: source_conversation_id,
                        message_id,
                    });
                }
                if original.protected_content {
                    return Err(EngineError::ProtectedContent);
                }

                // Forwarding is still a send into the destination conversation. Reuse the
                // canonical destination policy instead of letting the forward path bypass
                // membership, media/poll, community, channel, slow-mode, or secret-chat gates.
                let sender_is_participant = conversation
                    .participants
                    .iter()
                    .any(|participant| participant.actor_id == sender_id)
                    || conversation.owner_id.as_ref() == Some(&sender_id);
                if !sender_is_participant {
                    return Err(EngineError::SenderNotParticipant {
                        conversation_id: destination_conversation_id.clone(),
                        actor_id: sender_id.clone(),
                    });
                }
                if !conversation.permissions.can_send_messages {
                    return Err(EngineError::MessageSendPermissionDenied(
                        destination_conversation_id.clone(),
                    ));
                }
                if message_content_uses_media(&original.content)
                    && !conversation.permissions.can_send_media
                {
                    return Err(EngineError::MediaSendPermissionDenied(
                        destination_conversation_id.clone(),
                    ));
                }
                if matches!(&original.content, MessageContent::Poll { .. })
                    && !conversation.permissions.can_send_polls
                {
                    return Err(EngineError::PollSendPermissionDenied(
                        destination_conversation_id.clone(),
                    ));
                }
                if let Some(member) = self
                    .state
                    .communities
                    .get(&destination_conversation_id)
                    .and_then(|community| community.members.get(&sender_id))
                {
                    if matches!(member.status, MemberStatus::Left | MemberStatus::Banned)
                        || (matches!(member.status, MemberStatus::Restricted)
                            && member.restrictions.send_messages)
                    {
                        return Err(EngineError::CommunitySendRestricted(
                            destination_conversation_id.clone(),
                        ));
                    }
                    if message_content_uses_media(&original.content)
                        && matches!(member.status, MemberStatus::Restricted)
                        && member.restrictions.send_media
                    {
                        return Err(EngineError::CommunityMediaRestricted(
                            destination_conversation_id.clone(),
                        ));
                    }
                    if matches!(&original.content, MessageContent::Poll { .. })
                        && matches!(member.status, MemberStatus::Restricted)
                        && member.restrictions.send_polls
                    {
                        return Err(EngineError::CommunityPollRestricted(
                            destination_conversation_id.clone(),
                        ));
                    }
                }
                if matches!(conversation.kind, ConversationKind::Channel) {
                    let can_post =
                        conversation.owner_id.as_ref() == Some(&sender_id)
                            || conversation.participants.iter().any(|participant| {
                                participant.actor_id == sender_id
                                    && matches!(
                                        participant.role,
                                        ParticipantRole::Owner | ParticipantRole::Admin
                                    )
                            })
                            || self
                                .state
                                .communities
                                .get(&destination_conversation_id)
                                .is_some_and(|community| {
                                    community.members.get(&sender_id).is_some_and(|member| {
                                        matches!(member.status, MemberStatus::Administrator)
                                            && member.admin_rights.post_messages
                                    })
                                });
                    if !can_post {
                        return Err(EngineError::CommunitySendRestricted(
                            destination_conversation_id.clone(),
                        ));
                    }
                }
                if let Some(community) = self.state.communities.get(&destination_conversation_id) {
                    if let Some(thread_root) = &thread_root_message_id {
                        if let Some(topic_id) = topic_id_from_root(thread_root) {
                            let topic = community.topics.get(topic_id).ok_or_else(|| {
                                EngineError::ForumTopicNotFound {
                                    conversation_id: destination_conversation_id.clone(),
                                    topic_id: topic_id.to_string(),
                                }
                            })?;
                            if topic.closed || topic.hidden {
                                return Err(EngineError::ForumTopicClosed {
                                    conversation_id: destination_conversation_id.clone(),
                                    topic_id: topic_id.to_string(),
                                });
                            }
                        }
                    }
                    if let Some(seconds) = community.slow_mode_seconds {
                        let bypass = community.can_moderate(&sender_id);
                        if !bypass {
                            let latest_sender_message = self
                                .state
                                .messages
                                .get(&destination_conversation_id)
                                .into_iter()
                                .flat_map(|messages| messages.values())
                                .filter(|message| {
                                    message.sender_id == sender_id
                                        && !message.deleted
                                        && message
                                            .scheduled_at_ms
                                            .is_none_or(|scheduled| scheduled <= created_at_ms)
                                })
                                .max_by_key(|message| (message.created_at_ms, message.id.clone()));
                            if let Some(message) = latest_sender_message {
                                let retry_at = message
                                    .created_at_ms
                                    .saturating_add(i64::from(seconds).saturating_mul(1_000));
                                if created_at_ms < retry_at {
                                    return Err(EngineError::SlowModeActive {
                                        conversation_id: destination_conversation_id.clone(),
                                        retry_at_ms: retry_at,
                                    });
                                }
                            }
                        }
                    }
                }
                let is_secret_conversation = matches!(conversation.kind, ConversationKind::Secret);
                let is_secret_content = matches!(&original.content, MessageContent::Secret { .. });
                let is_service_content = matches!(&original.content, MessageContent::Service { .. });
                if is_secret_conversation && !is_secret_content && !is_service_content {
                    return Err(EngineError::SecretPlaintextRejected);
                }
                if !is_secret_conversation && is_secret_content {
                    return Err(EngineError::SecretContentOutsideSecretConversation);
                }
                if client_message_id.0.trim().is_empty() || client_message_id.0.len() > 200 {
                    return Err(EngineError::InvalidClientMessageId);
                }
                if self
                    .state
                    .messages
                    .get(&destination_conversation_id)
                    .is_some_and(|messages| messages.contains_key(&local_message_id))
                {
                    return Err(EngineError::DuplicateMessage {
                        conversation_id: destination_conversation_id,
                        message_id: local_message_id,
                    });
                }
                let privacy = privacy.normalized();
                let forward_origin = if privacy.drop_sender_names {
                    None
                } else {
                    Some(
                        original
                            .forward_origin
                            .clone()
                            .unwrap_or_else(|| {
                                format!("{}:{}", original.conversation_id.0, original.id.0)
                            }),
                    )
                };
                let mut content = original.content;
                if privacy.drop_captions {
                    content.clear_caption();
                }
                let message = Message {
                    id: local_message_id,
                    conversation_id: destination_conversation_id,
                    sender_id,
                    content,
                    reply_to_message_id: None,
                    thread_root_message_id,
                    forward_origin,
                    reply_markup: original.reply_markup,
                    reactions: Vec::new(),
                    delivery_state: DeliveryState::Pending { client_message_id },
                    created_at_ms,
                    edited_at_ms: None,
                    scheduled_at_ms,
                    silent,
                    protected_content: original.protected_content,
                    pinned: false,
                    deleted: false,
                };
                Ok(vec![Event::MessageQueued { message }])
            }
            Command::AcknowledgeMessage {
                conversation_id,
                local_message_id,
                server_message_id,
                accepted_at_ms,
            } => {
                self.require_message(&conversation_id, &local_message_id)?;
                if local_message_id != server_message_id
                    && self
                        .state
                        .messages
                        .get(&conversation_id)
                        .is_some_and(|messages| messages.contains_key(&server_message_id))
                {
                    return Err(EngineError::DuplicateMessage {
                        conversation_id,
                        message_id: server_message_id,
                    });
                }
                Ok(vec![Event::MessageAcknowledged {
                    conversation_id,
                    local_message_id,
                    server_message_id,
                    accepted_at_ms,
                }])
            }
            Command::SetDeliveryState {
                conversation_id,
                message_id,
                state,
            } => {
                self.require_message(&conversation_id, &message_id)?;
                Ok(vec![Event::DeliveryStateUpdated {
                    conversation_id,
                    message_id,
                    state,
                }])
            }
            Command::EditMessage {
                conversation_id,
                message_id,
                content,
                edited_at_ms,
            } => {
                self.require_message(&conversation_id, &message_id)?;
                Ok(vec![Event::MessageEdited {
                    conversation_id,
                    message_id,
                    content,
                    edited_at_ms,
                }])
            }
            Command::DeleteMessages {
                conversation_id,
                message_ids,
            } => {
                let unique: BTreeSet<_> = message_ids.into_iter().collect();
                if unique.is_empty() {
                    return Err(EngineError::EmptyMessageList);
                }
                for id in &unique {
                    self.require_message(&conversation_id, id)?;
                }
                Ok(vec![Event::MessagesDeleted {
                    conversation_id,
                    message_ids: unique.into_iter().collect(),
                }])
            }
            Command::MarkRead {
                conversation_id,
                actor_id,
                message_id,
            } => {
                self.require_actor(&actor_id)?;
                let conversation = self.require_conversation(&conversation_id)?;
                let has_access = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .map(|community| {
                        community_has_access(community, &actor_id)
                            || conversation.owner_id.as_ref() == Some(&actor_id)
                    })
                    .unwrap_or_else(|| {
                        conversation.owner_id.as_ref() == Some(&actor_id)
                            || conversation
                                .participants
                                .iter()
                                .any(|participant| participant.actor_id == actor_id)
                    });
                if !has_access {
                    return Err(EngineError::CommunityAccessDenied {
                        conversation_id,
                        actor_id,
                    });
                }
                self.require_message(&conversation_id, &message_id)?;
                Ok(vec![Event::ConversationRead {
                    conversation_id,
                    actor_id,
                    message_id,
                }])
            }
            Command::MarkTopicRead {
                conversation_id,
                topic_id,
                actor_id,
                message_id,
            } => {
                self.require_actor(&actor_id)?;
                let community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                if !community_has_access(community, &actor_id)
                    && self
                        .state
                        .conversations
                        .get(&conversation_id)
                        .and_then(|conversation| conversation.owner_id.as_ref())
                        != Some(&actor_id)
                {
                    return Err(EngineError::CommunityAccessDenied {
                        conversation_id,
                        actor_id,
                    });
                }
                let topic = community.topics.get(&topic_id).ok_or_else(|| {
                    EngineError::ForumTopicNotFound {
                        conversation_id: conversation_id.clone(),
                        topic_id: topic_id.clone(),
                    }
                })?;
                let message = self.require_message(&conversation_id, &message_id)?;
                let belongs_to_topic = message
                    .thread_root_message_id
                    .as_ref()
                    .is_some_and(|root| topic_id_from_root(root) == Some(topic_id.as_str()))
                    || topic.last_message_id.as_deref() == Some(message_id.0.as_str());
                if !belongs_to_topic {
                    return Err(EngineError::TopicMessageMismatch { topic_id });
                }
                Ok(vec![Event::TopicReadChanged {
                    conversation_id,
                    topic_id,
                    actor_id,
                    message_id,
                }])
            }
            Command::ReconcileSavedSublistParentAccess {
                conversation_id,
                actor_id,
                allowed,
            } => {
                self.require_actor(&actor_id)?;
                let conversation = self.require_conversation(&conversation_id)?;
                if !matches!(conversation.kind, ConversationKind::Group | ConversationKind::Channel)
                    || !self.state.communities.contains_key(&conversation_id)
                {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                if allowed {
                    let community = self
                        .state
                        .communities
                        .get(&conversation_id)
                        .expect("community existence checked");
                    let has_access = community_has_access(community, &actor_id)
                        || conversation.owner_id.as_ref() == Some(&actor_id)
                        || conversation
                            .participants
                            .iter()
                            .any(|participant| participant.actor_id == actor_id);
                    if !has_access {
                        return Err(EngineError::CommunityAccessDenied {
                            conversation_id,
                            actor_id,
                        });
                    }
                }
                Ok(vec![Event::SavedSublistParentAccessReconciled {
                    conversation_id,
                    actor_id,
                    allowed,
                }])
            }
            Command::ReconcileSavedSublistMembership {
                destination,
                actor_id,
                message_ids,
            } => {
                if !matches!(
                    &destination.child,
                    Some(ConversationChildIdentity::SavedSublist { .. })
                ) {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                // Reuse the canonical parent/actor authorization contract without
                // creating a renderer-visible draft mutation.
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                for message_id in &message_ids {
                    self.require_message(
                        destination_message_conversation_id(&destination),
                        message_id,
                    )?;
                }
                let message_ids = message_ids
                    .into_iter()
                    .map(|message_id| message_id.0)
                    .collect::<Vec<_>>();
                let mut validation = ConversationChildRuntimeState::new(
                    destination.clone(),
                    actor_id.clone(),
                )
                .ok_or(EngineError::InvalidConversationChildDestination)?;
                if !validation.reconcile_authoritative_message_ids(message_ids.clone()) {
                    return Err(EngineError::ConversationChildMessageMismatch);
                }
                Ok(vec![Event::SavedSublistMembershipReconciled {
                    destination,
                    actor_id,
                    message_ids,
                }])
            }
            Command::MarkConversationChildRead {
                destination,
                actor_id,
                message_id,
            } => {
                if !destination.is_valid() {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                self.require_actor(&actor_id)?;
                let conversation_id = destination.conversation_id.clone();
                let conversation = self.require_conversation(&conversation_id)?;
                let community = self.state.communities.get(&conversation_id);
                let has_access = community
                    .map(|community| {
                        community_has_access(community, &actor_id)
                            || conversation.owner_id.as_ref() == Some(&actor_id)
                    })
                    .unwrap_or_else(|| {
                        conversation.owner_id.as_ref() == Some(&actor_id)
                            || conversation
                                .participants
                                .iter()
                                .any(|participant| participant.actor_id == actor_id)
                    });
                if !has_access {
                    return Err(EngineError::CommunityAccessDenied {
                        conversation_id: conversation_id.clone(),
                        actor_id: actor_id.clone(),
                    });
                }
                match destination.child.as_ref() {
                    Some(ConversationChildIdentity::Topic { root_message_id }) => {
                        let topic_exists = community.is_some_and(|community| {
                            community.topics.contains_key(root_message_id)
                                || topic_id_from_root(&MessageId(root_message_id.clone()))
                                    .is_some_and(|topic_id| community.topics.contains_key(topic_id))
                        }) || self
                            .state
                            .messages
                            .get(&conversation_id)
                            .is_some_and(|messages| messages.contains_key(&MessageId(root_message_id.clone())));
                        if !topic_exists {
                            return Err(EngineError::InvalidConversationChildDestination);
                        }
                    }
                    Some(ConversationChildIdentity::SavedSublist { participant_id }) => {
                        // SavedSublist is valid only under self SavedMessages or an
                        // explicit server/native-authoritative parent relation for this
                        // account. Community-ness alone is never sufficient.
                        let self_saved_messages = matches!(
                            conversation.kind,
                            ConversationKind::SavedMessages
                        ) && conversation.owner_id.as_ref() == Some(&actor_id);
                        let server_parent = self
                            .state
                            .saved_sublist_parent_access
                            .get(&conversation_id)
                            .is_some_and(|actors| actors.contains(&actor_id));
                        let participant_exists = self.state.actors.contains_key(participant_id);
                        if (!self_saved_messages && !server_parent) || !participant_exists {
                            return Err(EngineError::InvalidConversationChildDestination);
                        }
                        let authoritative_membership = self
                            .state
                            .child_state(&destination, &actor_id)
                            .is_some_and(|child| {
                                child.has_authoritative_message(message_id.0.as_str())
                            });
                        if !authoritative_membership {
                            return Err(EngineError::ConversationChildMessageMismatch);
                        }
                    }
                    Some(ConversationChildIdentity::Conversation {
                        conversation_id: child_conversation_id,
                    }) => {
                        let child = self.require_conversation(child_conversation_id)?;
                        let child_access = child.owner_id.as_ref() == Some(&actor_id)
                            || child
                                .participants
                                .iter()
                                .any(|participant| participant.actor_id == actor_id)
                            || self
                                .state
                                .communities
                                .get(child_conversation_id)
                                .is_some_and(|community| community_has_access(community, &actor_id));
                        if !child_access {
                            return Err(EngineError::CommunityAccessDenied {
                                conversation_id: child_conversation_id.clone(),
                                actor_id: actor_id.clone(),
                            });
                        }
                    }
                    None => {}
                }
                let message_conversation_id = destination_message_conversation_id(&destination);
                let message = self.require_message(message_conversation_id, &message_id)?;
                if let Some(ConversationChildIdentity::Topic { root_message_id }) =
                    &destination.child
                {
                    let belongs = message
                        .thread_root_message_id
                        .as_ref()
                        .is_some_and(|root| root.0 == *root_message_id)
                        || message.id.0 == *root_message_id;
                    if !belongs {
                        return Err(EngineError::ConversationChildMessageMismatch);
                    }
                }
                Ok(vec![Event::ConversationChildReadChanged {
                    destination,
                    actor_id,
                    message_id,
                }])
            }
            Command::SetTopicDraft { draft } => {
                self.require_actor(&draft.actor_id)?;
                let community = self
                    .state
                    .communities
                    .get(&draft.conversation_id)
                    .ok_or_else(|| EngineError::CommunityNotFound(draft.conversation_id.clone()))?;
                if !community_has_access(community, &draft.actor_id)
                    && self
                        .state
                        .conversations
                        .get(&draft.conversation_id)
                        .and_then(|conversation| conversation.owner_id.as_ref())
                        != Some(&draft.actor_id)
                {
                    return Err(EngineError::CommunityAccessDenied {
                        conversation_id: draft.conversation_id,
                        actor_id: draft.actor_id,
                    });
                }
                if draft.topic_id.trim().is_empty()
                    || !community.topics.contains_key(&draft.topic_id)
                {
                    return Err(EngineError::ForumTopicNotFound {
                        conversation_id: draft.conversation_id,
                        topic_id: draft.topic_id,
                    });
                }
                Ok(vec![Event::TopicDraftChanged { draft }])
            }
            Command::SetConversationChildDraft {
                destination,
                actor_id,
                text,
                reply_to_message_id,
                updated_at_ms,
            } => {
                if !destination.is_valid() {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                self.require_actor(&actor_id)?;
                let conversation_id = destination.conversation_id.clone();
                let conversation = self.require_conversation(&conversation_id)?;
                let community = self.state.communities.get(&conversation_id);
                let has_access = community
                    .map(|community| {
                        community_has_access(community, &actor_id)
                            || conversation.owner_id.as_ref() == Some(&actor_id)
                    })
                    .unwrap_or_else(|| {
                        conversation.owner_id.as_ref() == Some(&actor_id)
                            || conversation
                                .participants
                                .iter()
                                .any(|participant| participant.actor_id == actor_id)
                    });
                if !has_access {
                    return Err(EngineError::CommunityAccessDenied {
                        conversation_id: conversation_id.clone(),
                        actor_id: actor_id.clone(),
                    });
                }
                match destination.child.as_ref() {
                    Some(ConversationChildIdentity::Topic { root_message_id }) => {
                        let topic_exists = community.is_some_and(|community| {
                            community.topics.contains_key(root_message_id)
                                || topic_id_from_root(&MessageId(root_message_id.clone()))
                                    .is_some_and(|topic_id| community.topics.contains_key(topic_id))
                        }) || self
                            .state
                            .messages
                            .get(&conversation_id)
                            .is_some_and(|messages| messages.contains_key(&MessageId(root_message_id.clone())));
                        if !topic_exists {
                            return Err(EngineError::InvalidConversationChildDestination);
                        }
                    }
                    Some(ConversationChildIdentity::SavedSublist { participant_id }) => {
                        // SavedSublist is valid only under self SavedMessages or an
                        // explicit server/native-authoritative parent relation for this
                        // account. Community-ness alone is never sufficient.
                        let self_saved_messages = matches!(
                            conversation.kind,
                            ConversationKind::SavedMessages
                        ) && conversation.owner_id.as_ref() == Some(&actor_id);
                        let server_parent = self
                            .state
                            .saved_sublist_parent_access
                            .get(&conversation_id)
                            .is_some_and(|actors| actors.contains(&actor_id));
                        let participant_exists = self.state.actors.contains_key(participant_id);
                        if (!self_saved_messages && !server_parent) || !participant_exists {
                            return Err(EngineError::InvalidConversationChildDestination);
                        }
                    }
                    Some(ConversationChildIdentity::Conversation {
                        conversation_id: child_conversation_id,
                    }) => {
                        let child = self.require_conversation(child_conversation_id)?;
                        let child_access = child.owner_id.as_ref() == Some(&actor_id)
                            || child
                                .participants
                                .iter()
                                .any(|participant| participant.actor_id == actor_id)
                            || self
                                .state
                                .communities
                                .get(child_conversation_id)
                                .is_some_and(|community| community_has_access(community, &actor_id));
                        if !child_access {
                            return Err(EngineError::CommunityAccessDenied {
                                conversation_id: child_conversation_id.clone(),
                                actor_id: actor_id.clone(),
                            });
                        }
                    }
                    None => {}
                }
                if let Some(reply_to_message_id) = &reply_to_message_id {
                    self.require_message(
                        destination_message_conversation_id(&destination),
                        reply_to_message_id,
                    )?;
                    if matches!(
                        &destination.child,
                        Some(ConversationChildIdentity::SavedSublist { .. })
                    ) {
                        let authoritative_membership = self
                            .state
                            .child_state(&destination, &actor_id)
                            .is_some_and(|child| {
                                child.has_authoritative_message(reply_to_message_id.0.as_str())
                            });
                        if !authoritative_membership {
                            return Err(EngineError::ConversationChildMessageMismatch);
                        }
                    }
                }
                Ok(vec![Event::ConversationChildDraftChanged {
                    destination,
                    actor_id,
                    text,
                    reply_to_message_id: reply_to_message_id.map(|id| id.0),
                    updated_at_ms,
                }])
            }
            Command::ReplaceConversationChildWindow {
                destination,
                actor_id,
                message_ids,
                skipped_before,
                skipped_after,
                full_count,
            } => {
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                for message_id in &message_ids {
                    self.decide(Command::MarkConversationChildRead {
                        destination: destination.clone(),
                        actor_id: actor_id.clone(),
                        message_id: message_id.clone(),
                    })?;
                }
                let message_ids = message_ids
                    .into_iter()
                    .map(|message_id| message_id.0)
                    .collect::<Vec<_>>();
                let mut pagination = crate::conversation::ConversationChildPaginationState::default();
                if !pagination.replace_window(
                    message_ids.clone(),
                    skipped_before,
                    skipped_after,
                    full_count,
                ) {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                Ok(vec![Event::ConversationChildWindowReplaced {
                    destination,
                    actor_id,
                    message_ids,
                    skipped_before,
                    skipped_after,
                    full_count,
                }])
            }
            Command::SetConversationChildPinned {
                destination,
                actor_id,
                pinned,
            } => {
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                Ok(vec![Event::ConversationChildPinnedChanged {
                    destination,
                    actor_id,
                    pinned,
                }])
            }
            Command::SetConversationChildActive {
                destination,
                actor_id,
                active,
            } => {
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                Ok(vec![Event::ConversationChildActiveChanged {
                    destination,
                    actor_id,
                    active,
                }])
            }
            Command::SetConversationChildMarkedUnread {
                destination,
                actor_id,
                marked_unread,
            } => {
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                let conversation = self.require_conversation(&destination.conversation_id)?;
                let currently_unread = self
                    .state
                    .conversation_child_states
                    .iter()
                    .find(|state| state.destination == destination && state.actor_id == actor_id)
                    .is_some_and(|state| state.marked_unread || state.unread_count.unwrap_or(0) > 0);
                let context = crate::conversation::ConversationChildUnreadContext {
                    parent_is_self: matches!(conversation.kind, ConversationKind::SavedMessages)
                        && conversation.owner_id.as_ref() == Some(&actor_id),
                    parent_is_community: self.state.communities.contains_key(&destination.conversation_id),
                    actor_is_monoforum_admin: false,
                };
                if !destination.can_toggle_unread(currently_unread, context) {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                Ok(vec![Event::ConversationChildMarkedUnreadChanged {
                    destination,
                    actor_id,
                    marked_unread,
                }])
            }
            Command::ReconcileConversationChildUnreadThings {
                destination,
                actor_id,
                known,
                mention_message_ids,
                reaction_message_ids,
                poll_vote_message_ids,
                pending_incoming_notification_message_ids,
            } => {
                if destination.child.is_none() {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                // Reuse the canonical destination authorization/identity contract
                // without producing a draft mutation.
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;

                let mut owned_ids = pending_incoming_notification_message_ids.clone();
                if known {
                    owned_ids.extend(mention_message_ids.iter().cloned());
                    owned_ids.extend(reaction_message_ids.iter().cloned());
                    owned_ids.extend(poll_vote_message_ids.iter().cloned());
                }

                for message_id in &owned_ids {
                    self.decide(Command::MarkConversationChildRead {
                        destination: destination.clone(),
                        actor_id: actor_id.clone(),
                        message_id: message_id.clone(),
                    })?;
                }

                let mut unread_things = ConversationChildUnreadThings::default();
                unread_things.reconcile(
                    known,
                    mention_message_ids.into_iter().map(|id| id.0).collect(),
                    reaction_message_ids.into_iter().map(|id| id.0).collect(),
                    poll_vote_message_ids.into_iter().map(|id| id.0).collect(),
                );
                let mut pending = ConversationChildUnreadThings::default();
                pending.reconcile(
                    true,
                    pending_incoming_notification_message_ids
                        .into_iter()
                        .map(|id| id.0)
                        .collect(),
                    Vec::new(),
                    Vec::new(),
                );

                Ok(vec![Event::ConversationChildUnreadThingsReconciled {
                    destination,
                    actor_id,
                    unread_things,
                    pending_incoming_notification_message_ids: pending.mention_message_ids,
                }])
            }
            Command::SetConversationChildNoPaidMessages {
                destination,
                actor_id,
                no_paid_messages,
            } => {
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                if !matches!(
                    &destination.child,
                    Some(ConversationChildIdentity::SavedSublist { .. })
                ) {
                    return Err(EngineError::InvalidConversationChildDestination);
                }
                Ok(vec![Event::ConversationChildNoPaidMessagesChanged {
                    destination,
                    actor_id,
                    no_paid_messages,
                }])
            }
            Command::DestroyConversationChild {
                destination,
                actor_id,
            } => {
                self.decide(Command::SetConversationChildDraft {
                    destination: destination.clone(),
                    actor_id: actor_id.clone(),
                    text: String::new(),
                    reply_to_message_id: None,
                    updated_at_ms: 0,
                })?;
                Ok(vec![Event::ConversationChildDestroyed {
                    destination,
                    actor_id,
                }])
            }
            Command::SetReaction {
                conversation_id,
                message_id,
                reaction,
            } => {
                self.require_message(&conversation_id, &message_id)?;
                if reaction.reaction.trim().is_empty() {
                    return Err(EngineError::InvalidClientMessageId);
                }
                Ok(vec![Event::ReactionUpdated {
                    conversation_id,
                    message_id,
                    reaction,
                }])
            }
            Command::PinMessage {
                conversation_id,
                message_id,
                pinned,
            } => {
                self.require_message(&conversation_id, &message_id)?;
                Ok(vec![Event::MessagePinned {
                    conversation_id,
                    message_id,
                    pinned,
                }])
            }
            Command::VotePoll {
                conversation_id,
                message_id,
                actor_id,
                option_ids,
            } => {
                self.require_actor(&actor_id)?;
                let message = self.require_message(&conversation_id, &message_id)?;
                let (valid_ids, multiple_answers) = match &message.content {
                    MessageContent::Poll {
                        options,
                        multiple_answers,
                        ..
                    } => (
                        options
                            .iter()
                            .map(|option| option.id.clone())
                            .collect::<BTreeSet<_>>(),
                        *multiple_answers,
                    ),
                    _ => return Err(EngineError::InvalidPollVote),
                };
                let selected = option_ids.into_iter().collect::<BTreeSet<_>>();
                if (!multiple_answers && selected.len() > 1)
                    || selected.iter().any(|id| !valid_ids.contains(id))
                {
                    return Err(EngineError::InvalidPollVote);
                }
                Ok(vec![Event::PollVoteChanged {
                    conversation_id,
                    message_id,
                    actor_id,
                    option_ids: selected.into_iter().collect(),
                }])
            }
            Command::CreateInvoice { invoice } => {
                if !invoice.is_valid() {
                    return Err(EngineError::InvalidInvoice);
                }
                self.require_actor(&invoice.seller_id)?;
                self.require_conversation(&invoice.conversation_id)?;
                Ok(vec![Event::InvoiceCreated { invoice }])
            }
            Command::UpsertOrder { order } => {
                if !self.state.invoices.contains_key(&order.invoice_id) {
                    return Err(EngineError::InvoiceNotFound(order.invoice_id));
                }
                self.require_actor(&order.buyer_id)?;
                Ok(vec![Event::OrderUpserted { order }])
            }
            Command::CheckoutInvoice {
                invoice_id,
                order_id,
                buyer_id,
                customer,
                created_at_ms,
            } => {
                let invoice = self
                    .state
                    .invoices
                    .get(&invoice_id)
                    .cloned()
                    .ok_or_else(|| EngineError::InvoiceNotFound(invoice_id.clone()))?;
                self.require_actor(&buyer_id)?;
                if invoice
                    .expires_at_ms
                    .is_some_and(|expires_at_ms| expires_at_ms <= created_at_ms)
                {
                    return Err(EngineError::InvoiceExpired(invoice_id));
                }
                if !invoice.customer_information_is_valid(customer.as_ref()) {
                    return Err(EngineError::InvalidCustomerInfo);
                }
                if let Some(existing) = self.state.orders.get(&order_id) {
                    if existing.invoice_id == invoice.id && existing.buyer_id == buyer_id {
                        return Ok(vec![Event::OrderUpserted {
                            order: existing.clone(),
                        }]);
                    }
                    return Err(EngineError::OrderConflict(order_id));
                }
                let amount_minor = invoice
                    .checked_total_minor()
                    .ok_or(EngineError::InvalidInvoice)?;
                if amount_minor <= 0 {
                    return Err(EngineError::InvalidInvoice);
                }
                let buyer_account = wallet_account_id(&buyer_id);
                let seller_account = wallet_account_id(&invoice.seller_id);
                let mut wallet = self.state.wallet.clone();
                ensure_wallet_account(&mut wallet, &buyer_account, &buyer_id, created_at_ms)?;
                ensure_wallet_account(
                    &mut wallet,
                    &seller_account,
                    &invoice.seller_id,
                    created_at_ms,
                )?;
                let entry = wallet.transfer(
                    format!("checkout:{order_id}"),
                    &buyer_account,
                    &seller_account,
                    Money::new(&invoice.currency, amount_minor),
                    Some(invoice.id.clone()),
                    created_at_ms,
                )?;
                let order = PaymentOrder {
                    id: order_id,
                    invoice_id: invoice.id,
                    buyer_id,
                    status: PaymentStatus::Paid,
                    amount: Money::new(&invoice.currency, amount_minor),
                    customer,
                    provider_payment_id: Some(entry.id.clone()),
                    provider_receipt_url: Some(format!("fabushi://payments/receipts/{}", entry.id)),
                    created_at_ms,
                    updated_at_ms: created_at_ms,
                };
                Ok(vec![
                    Event::WalletChanged { wallet, entry },
                    Event::OrderUpserted { order },
                ])
            }
            Command::RefundOrder {
                order_id,
                seller_id,
                request_id,
                refunded_at_ms,
            } => {
                let mut order = self
                    .state
                    .orders
                    .get(&order_id)
                    .cloned()
                    .ok_or_else(|| EngineError::OrderNotFound(order_id.clone()))?;
                let invoice = self
                    .state
                    .invoices
                    .get(&order.invoice_id)
                    .cloned()
                    .ok_or_else(|| EngineError::InvoiceNotFound(order.invoice_id.clone()))?;
                if invoice.seller_id != seller_id {
                    return Err(EngineError::RefundForbidden(order_id));
                }
                if order.status == PaymentStatus::Refunded {
                    return Ok(vec![Event::OrderUpserted { order }]);
                }
                if order.status != PaymentStatus::Paid {
                    return Err(EngineError::OrderNotRefundable(order_id));
                }
                let original_entry_id = order
                    .provider_payment_id
                    .clone()
                    .ok_or_else(|| EngineError::OrderNotRefundable(order.id.clone()))?;
                let mut wallet = self.state.wallet.clone();
                let entry =
                    wallet.refund_transfer(request_id, &original_entry_id, refunded_at_ms)?;
                order.status = PaymentStatus::Refunded;
                order.updated_at_ms = refunded_at_ms;
                Ok(vec![
                    Event::WalletChanged { wallet, entry },
                    Event::OrderUpserted { order },
                ])
            }
            Command::CreditWalletSettlement {
                request_id,
                owner_id,
                amount,
                reference,
                settled_at_ms,
            } => {
                self.require_actor(&owner_id)?;
                let account_id = wallet_account_id(&owner_id);
                let mut wallet = self.state.wallet.clone();
                ensure_wallet_account(&mut wallet, &account_id, &owner_id, settled_at_ms)?;
                let entry =
                    wallet.credit(request_id, &account_id, amount, reference, settled_at_ms)?;
                Ok(vec![Event::WalletChanged { wallet, entry }])
            }
            Command::SetWalletFiatCurrency { currency } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.rates.set_currency(&currency)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ApplyWalletRateSnapshot {
                rates,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.rates.apply_snapshot(rates, observed_at_ms)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::MarkWalletRateRefreshFailed { observed_at_ms } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.rates.mark_refresh_failure(observed_at_ms);
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::BeginWalletFunding {
                address,
                asset,
                base_currency,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .onramp
                    .begin(address, asset, base_currency.as_deref())?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ResolveWalletFundingProvider {
                request_id,
                providers,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.onramp.resolve_provider(request_id, &providers)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::CompleteWalletFunding {
                request_id,
                session_url,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .onramp
                    .complete(request_id, session_url.as_deref())?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::CancelWalletFunding { request_id } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.onramp.cancel(request_id)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ShowWalletPanel => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.panel.show();
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::MinimizeWalletPanel => {
                let mut runtime = self.state.wallet.runtime.clone();
                if !runtime.panel.minimize() {
                    return Err(EngineError::WalletPanelNotVisible);
                }
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::CloseWalletPanel => {
                let mut runtime = self.state.wallet.runtime.clone();
                if !runtime.panel.close() {
                    return Err(EngineError::WalletPanelNotVisible);
                }
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::SetWalletTransactionsVisible { visible } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.panel.set_transactions_visible(visible);
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::JournalOutboundTransfer { record } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.outbound_transfers.prepare(record)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::JournalQuotedOutboundTransfer {
                record,
                quote,
                balance_nano,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.journal_quoted_outbound_transfer(
                    record,
                    &quote,
                    balance_nano,
                    observed_at_ms,
                )?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::MarkOutboundTransferHandoff {
                record_id,
                message_token,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .outbound_transfers
                    .mark_handoff_possible(&record_id, message_token)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::RecordOutboundTransferLookup { record_id } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.outbound_transfers.note_lookup_attempt(&record_id)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::StopOutboundTransferLookup { record_id } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.outbound_transfers.stop_lookup(&record_id)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::SettleOutboundTransfer {
                record_id,
                terminal,
                confirmed_hash,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .outbound_transfers
                    .settle(&record_id, terminal, confirmed_hash)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ReconcileWalletUserAddress {
                actor_id,
                serial,
                address,
                public_key,
            } => {
                self.require_actor(&actor_id)?;
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .address_directory
                    .apply_user_answer(actor_id, serial, address, public_key)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ReconcileWalletAddressOwner {
                address,
                actor_id,
                public_key,
            } => {
                if let Some(actor_id) = &actor_id {
                    self.require_actor(actor_id)?;
                }
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .address_directory
                    .apply_owner_answer(address, actor_id, public_key)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::SetWalletAddressServiceUnavailable { unavailable } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .address_directory
                    .set_service_unavailable(unavailable);
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::BeginWalletLiveGeneration => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.live.begin_generation()?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ReconcileWalletLivePresence {
                generation,
                presence,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .live
                    .apply_presence(generation, presence, observed_at_ms)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::MarkWalletLiveStateFailed {
                generation,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .live
                    .note_state_failure(generation, observed_at_ms)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::MarkWalletStreamResynced {
                generation,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .live
                    .mark_stream_resync(generation, observed_at_ms)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::SpendWalletHistoryPageRequest { generation } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.live.spend_history_page_request(generation)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::RecordWalletHistoryProgress {
                generation,
                visible_rows,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.live.note_history_progress(generation, visible_rows)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::RearmWalletHistoryWalk { generation } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.live.rearm_history_walk(generation)?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::BeginWalletSponsoredFeeRequest {
                network_generation,
                identity,
                transfer_min_nano,
                configured_min_nano,
                observed_at_ms,
                force,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.sponsored_fees.begin_request(
                    network_generation,
                    identity,
                    transfer_min_nano,
                    configured_min_nano,
                    observed_at_ms,
                    force,
                )?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ApplyWalletSponsoredFeeInfo {
                serial,
                network_generation,
                identity,
                info,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.sponsored_fees.apply_info(
                    serial,
                    network_generation,
                    &identity,
                    info,
                    observed_at_ms,
                )?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::FailWalletSponsoredFeeRequest {
                serial,
                network_generation,
                identity,
                observed_at_ms,
            } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime.sponsored_fees.fail_request(
                    serial,
                    network_generation,
                    &identity,
                    observed_at_ms,
                )?;
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::ResetWalletSponsoredFeeGeneration { network_generation } => {
                let mut runtime = self.state.wallet.runtime.clone();
                runtime
                    .sponsored_fees
                    .reset_for_network_generation(network_generation);
                Ok(vec![Event::WalletRuntimeChanged { runtime }])
            }
            Command::UpsertConnectedAppSession { session } => {
                let mut state = self.state.connected_apps.clone();
                state.upsert_session(session)?;
                Ok(vec![Event::ConnectedAppStateChanged { state }])
            }
            Command::QueueConnectedAppRequest {
                request,
                observed_at_ms,
            } => {
                let mut state = self.state.connected_apps.clone();
                state.queue_request(request, observed_at_ms)?;
                Ok(vec![Event::ConnectedAppStateChanged { state }])
            }
            Command::ResolveConnectedAppRequest {
                session_id,
                request_id,
                decision,
                operation_id,
                signed_payload,
                answer,
                observed_at_ms,
            } => {
                let mut state = self.state.connected_apps.clone();
                state.record_claim(
                    session_id,
                    &request_id,
                    decision,
                    operation_id,
                    signed_payload,
                    answer,
                    observed_at_ms,
                )?;
                Ok(vec![Event::ConnectedAppStateChanged { state }])
            }
            Command::ResolveConnectedAppWalletRequest {
                session_id,
                request_id,
                decision,
                wallet_identity,
                operation_id,
                signed_payload,
                answer,
                observed_at_ms,
            } => {
                let mut state = self.state.connected_apps.clone();
                state.record_wallet_claim(
                    session_id,
                    &request_id,
                    decision,
                    wallet_identity,
                    operation_id,
                    signed_payload,
                    answer,
                    observed_at_ms,
                )?;
                Ok(vec![Event::ConnectedAppStateChanged { state }])
            }
            Command::CloseConnectedAppSession {
                session_id,
                closed_at_ms,
            } => {
                let mut state = self.state.connected_apps.clone();
                state.close_session(session_id, closed_at_ms)?;
                Ok(vec![Event::ConnectedAppStateChanged { state }])
            }
            Command::PruneConnectedAppClaims { observed_at_ms } => {
                let mut state = self.state.connected_apps.clone();
                state.prune_claims(observed_at_ms);
                Ok(vec![Event::ConnectedAppStateChanged { state }])
            }
            Command::ReconcileEntitlement { entitlement } => {
                self.require_actor(&entitlement.owner_id)?;
                if entitlement.id.trim().is_empty()
                    || entitlement.product_id.trim().is_empty()
                    || entitlement.starts_at_ms < 0
                    || entitlement.expires_at_ms.is_some_and(|value| value <= entitlement.starts_at_ms)
                {
                    return Err(EngineError::InvalidStoryStealthRequest);
                }
                Ok(vec![Event::EntitlementReconciled { entitlement }])
            }
            Command::ActivateStoryStealth {
                actor_id,
                request_id,
                activated_at_ms,
            } => {
                self.require_actor(&actor_id)?;
                if request_id.trim().is_empty() {
                    return Err(EngineError::InvalidStoryStealthRequest);
                }
                let current = self
                    .state
                    .story_stealth
                    .get(&actor_id)
                    .cloned()
                    .unwrap_or_default();
                if current.last_activation_request_id.as_deref() == Some(request_id.as_str())
                    || current.enabled_at(activated_at_ms)
                {
                    return Ok(vec![Event::StoryStealthChanged {
                        actor_id,
                        state: current,
                    }]);
                }
                if current.cooling_down_at(activated_at_ms) {
                    return Err(EngineError::StoryStealthCooldown {
                        retry_at_ms: current.cooldown_till_ms,
                    });
                }
                let entitled = self.state.entitlements.values().any(|entitlement| {
                    entitlement.is_active_for(
                        &actor_id,
                        STORY_STEALTH_PRODUCT_ID,
                        activated_at_ms,
                    )
                });
                if !entitled {
                    return Err(EngineError::StoryStealthEntitlementRequired);
                }
                let state = StoryStealthState {
                    enabled_till_ms: activated_at_ms.saturating_add(STORY_STEALTH_ACTIVE_MS),
                    cooldown_till_ms: activated_at_ms.saturating_add(STORY_STEALTH_COOLDOWN_MS),
                    last_activation_request_id: Some(request_id),
                };
                let since_ms = activated_at_ms.saturating_sub(STORY_STEALTH_RETROACTIVE_MS);
                let mut events = self
                    .state
                    .stories
                    .values()
                    .filter_map(|story| {
                        let mut story = story.clone();
                        story
                            .anonymize_recent_view(&actor_id, since_ms)
                            .then_some(Event::StoryChanged { story })
                    })
                    .collect::<Vec<_>>();
                events.push(Event::StoryStealthChanged { actor_id, state });
                Ok(events)
            }
            Command::PublishStory { actor_id, story } => {
                self.require_actor(&actor_id)?;
                if story.owner_id != actor_id
                    || story.id.0.trim().is_empty()
                    || story.media.id.trim().is_empty()
                    || story.expires_at_ms <= story.created_at_ms
                {
                    return Err(EngineError::InvalidStory);
                }
                Ok(vec![Event::StoryChanged { story }])
            }
            Command::DeleteStory { actor_id, story_id } => {
                let story = self
                    .state
                    .stories
                    .get(&story_id)
                    .ok_or_else(|| EngineError::StoryNotFound(story_id.clone()))?;
                if story.owner_id != actor_id {
                    return Err(EngineError::StoryPermissionDenied(story_id));
                }
                Ok(vec![Event::StoryDeleted { story_id }])
            }
            Command::ViewStory {
                actor_id,
                story_id,
                viewed_at_ms,
            } => {
                self.require_actor(&actor_id)?;
                let mut story = self
                    .state
                    .stories
                    .get(&story_id)
                    .cloned()
                    .ok_or_else(|| EngineError::StoryNotFound(story_id.clone()))?;
                if !story.is_visible_to(&actor_id, false, false) {
                    return Err(EngineError::StoryPermissionDenied(story_id));
                }
                if self
                    .state
                    .story_stealth
                    .get(&actor_id)
                    .is_some_and(|state| state.enabled_at(viewed_at_ms))
                {
                    story.record_anonymous_view(viewed_at_ms)?;
                } else {
                    story.record_view(actor_id, viewed_at_ms)?;
                }
                Ok(vec![Event::StoryChanged { story }])
            }
            Command::ReactStory {
                actor_id,
                story_id,
                reaction,
                reacted_at_ms,
            } => {
                let mut story = self
                    .state
                    .stories
                    .get(&story_id)
                    .cloned()
                    .ok_or_else(|| EngineError::StoryNotFound(story_id.clone()))?;
                if !story.is_visible_to(&actor_id, false, false) {
                    return Err(EngineError::StoryPermissionDenied(story_id));
                }
                story.react(&actor_id, reaction, reacted_at_ms)?;
                Ok(vec![Event::StoryChanged { story }])
            }
            Command::UpdateCommunity {
                actor_id,
                mut community,
            } => {
                let conversation = self.require_conversation(&community.conversation_id)?;
                if !matches!(
                    conversation.kind,
                    crate::conversation::ConversationKind::Group
                        | crate::conversation::ConversationKind::Channel
                ) {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                if let Some(existing) = self.state.communities.get(&community.conversation_id) {
                    require_community_admin(existing, &actor_id, CommunityAdminAction::ChangeInfo)?;
                } else if conversation.owner_id.as_ref() != Some(&actor_id)
                    && !conversation.participants.iter().any(|participant| {
                        participant.actor_id == actor_id
                            && matches!(participant.role, crate::actor::ParticipantRole::Owner)
                    })
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                if let Some(existing) = self.state.communities.get(&community.conversation_id) {
                    // Membership, moderation, invites, topics and policy are server-owned.
                    // Only the community metadata above is client-editable through this command.
                    community.members = existing.members.clone();
                    community.invite_links = existing.invite_links.clone();
                    community.pending_join_requests = existing.pending_join_requests.clone();
                    community.topics = existing.topics.clone();
                    community.banned_words = existing.banned_words.clone();
                    community.slow_mode_seconds = existing.slow_mode_seconds;
                    community.subscribers = existing.subscribers.clone();
                    community.admin_log = existing.admin_log.clone();
                } else {
                    // Only the conversation owner may create the first community state.
                    // Seed that owner as the sole owner so a non-owner cannot squat the state.
                    community.subscribers.clear();
                    community.admin_log.clear();
                    let requested_members = std::mem::take(&mut community.members);
                    community.invite_links.clear();
                    community.pending_join_requests.clear();
                    community.topics.clear();
                    community.banned_words.clear();
                    community.members = conversation
                        .participants
                        .iter()
                        .map(|participant| {
                            let requested = requested_members.get(&participant.actor_id);
                            let (status, admin_rights) = match participant.role {
                                ParticipantRole::Owner => (
                                    MemberStatus::Owner,
                                    AdminRights {
                                        change_info: true,
                                        post_messages: true,
                                        edit_messages: true,
                                        delete_messages: true,
                                        ban_members: true,
                                        invite_members: true,
                                        pin_messages: true,
                                        manage_topics: true,
                                        manage_calls: true,
                                        add_admins: true,
                                        remain_anonymous: true,
                                    },
                                ),
                                ParticipantRole::Admin => (
                                    MemberStatus::Administrator,
                                    requested
                                        .map(|member| member.admin_rights.clone())
                                        .unwrap_or_default(),
                                ),
                                ParticipantRole::Restricted => (
                                    MemberStatus::Restricted,
                                    requested
                                        .map(|member| member.admin_rights.clone())
                                        .unwrap_or_default(),
                                ),
                                ParticipantRole::Member => {
                                    (MemberStatus::Member, AdminRights::default())
                                }
                            };
                            (
                                participant.actor_id.clone(),
                                CommunityMember {
                                    actor_id: participant.actor_id.clone(),
                                    status,
                                    admin_title: requested
                                        .and_then(|member| member.admin_title.clone()),
                                    admin_rights,
                                    restrictions: requested
                                        .map(|member| member.restrictions.clone())
                                        .unwrap_or_default(),
                                    joined_at_ms: participant.joined_at_ms,
                                    invited_by: requested
                                        .and_then(|member| member.invited_by.clone()),
                                },
                            )
                        })
                        .collect();
                }
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::SubscribeChannel {
                actor_id,
                conversation_id,
                subscribed_at_ms,
            } => {
                self.require_actor(&actor_id)?;
                let conversation = self.require_conversation(&conversation_id)?;
                if !matches!(conversation.kind, ConversationKind::Channel) {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                if !community
                    .public_username
                    .as_deref()
                    .is_some_and(|username| !username.trim().is_empty())
                    && !community.is_subscriber(&actor_id)
                    && !community.members.contains_key(&actor_id)
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let was_subscriber = community.is_subscriber(&actor_id);
                community.subscribe_channel(actor_id.clone(), subscribed_at_ms)?;
                if !was_subscriber {
                    append_community_audit(
                        &mut community,
                        &actor_id,
                        CommunityAuditAction::SubscriptionAdded,
                        Some(actor_id.clone()),
                        None,
                        None,
                        subscribed_at_ms,
                    );
                }
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::UnsubscribeChannel {
                actor_id,
                conversation_id,
                unsubscribed_at_ms,
            } => {
                self.require_actor(&actor_id)?;
                let conversation = self.require_conversation(&conversation_id)?;
                if !matches!(conversation.kind, ConversationKind::Channel) {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                if conversation.owner_id.as_ref() == Some(&actor_id) {
                    return Err(EngineError::Community(
                        CommunityError::OwnerCannotUnsubscribe,
                    ));
                }
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                let was_subscriber = community.is_subscriber(&actor_id);
                community.unsubscribe_channel(&actor_id)?;
                if was_subscriber {
                    append_community_audit(
                        &mut community,
                        &actor_id,
                        CommunityAuditAction::SubscriptionRemoved,
                        Some(actor_id.clone()),
                        None,
                        None,
                        unsubscribed_at_ms,
                    );
                }
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::SetCommunitySlowMode {
                actor_id,
                conversation_id,
                seconds,
                changed_at_ms,
            } => {
                let conversation = self.require_conversation(&conversation_id)?;
                if !matches!(
                    conversation.kind,
                    ConversationKind::Group | ConversationKind::Channel
                ) {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                if seconds.is_some_and(|seconds| seconds > 86_400) {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                require_community_admin(&community, &actor_id, CommunityAdminAction::ChangeInfo)?;
                community.slow_mode_seconds = seconds;
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::SlowModeChanged,
                    None,
                    Some("slowMode".into()),
                    None,
                    changed_at_ms,
                );
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::ModerateCommunityMember {
                actor_id,
                conversation_id,
                member,
                reason,
                decided_at_ms,
            } => {
                self.require_actor(&member.actor_id)?;
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                require_community_admin(&community, &actor_id, CommunityAdminAction::BanMembers)?;
                if member.actor_id == actor_id
                    || community
                        .members
                        .get(&member.actor_id)
                        .is_some_and(|existing| matches!(existing.status, MemberStatus::Owner))
                    || matches!(member.status, MemberStatus::Owner)
                    || matches!(member.status, MemberStatus::Administrator)
                    || community
                        .members
                        .get(&member.actor_id)
                        .is_some_and(|existing| {
                            matches!(existing.status, MemberStatus::Administrator)
                        })
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let target_actor_id = member.actor_id.clone();
                let target_id = format!("member:{}", target_actor_id.0);
                community.upsert_member(member.clone());
                if matches!(member.status, MemberStatus::Left | MemberStatus::Banned) {
                    community.subscribers.remove(&target_actor_id);
                }
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::MemberChanged,
                    Some(target_actor_id),
                    Some(target_id),
                    reason,
                    decided_at_ms,
                );
                let participant_event = self
                    .state
                    .conversations
                    .get(&conversation_id)
                    .is_some_and(|conversation| {
                        matches!(
                            conversation.kind,
                            ConversationKind::Group | ConversationKind::Channel
                        )
                    })
                    .then(|| {
                        community
                            .members
                            .get(&member.actor_id)
                            .and_then(participant_for_community_member)
                            .map(|participant| Event::ConversationParticipantUpserted {
                                conversation_id: conversation_id.clone(),
                                participant,
                            })
                            .unwrap_or_else(|| Event::ConversationParticipantRemoved {
                                conversation_id: conversation_id.clone(),
                                actor_id: member.actor_id.clone(),
                            })
                    });
                let mut events = vec![Event::CommunityChanged { community }];
                if let Some(event) = participant_event {
                    events.push(event);
                }
                Ok(events)
            }
            Command::SetCommunityMember {
                actor_id,
                conversation_id,
                member,
            } => {
                self.require_actor(&member.actor_id)?;
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                let caller_is_owner = is_community_owner(&community, &actor_id);
                let target_is_owner = community
                    .members
                    .get(&member.actor_id)
                    .is_some_and(|existing| matches!(existing.status, MemberStatus::Owner));
                let target_is_administrator = community
                    .members
                    .get(&member.actor_id)
                    .is_some_and(|existing| matches!(existing.status, MemberStatus::Administrator));
                if (target_is_owner || matches!(member.status, MemberStatus::Owner))
                    && !caller_is_owner
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let action = if target_is_owner || target_is_administrator {
                    CommunityAdminAction::AddAdmins
                } else {
                    match member.status {
                        MemberStatus::Owner | MemberStatus::Administrator => {
                            CommunityAdminAction::AddAdmins
                        }
                        MemberStatus::Restricted | MemberStatus::Left | MemberStatus::Banned => {
                            CommunityAdminAction::BanMembers
                        }
                        MemberStatus::Member => CommunityAdminAction::InviteMembers,
                    }
                };
                require_community_admin(&community, &actor_id, action)?;
                let target_actor_id = member.actor_id.clone();
                let joined_at_ms = member.joined_at_ms;
                community.upsert_member(member);
                if matches!(
                    community
                        .members
                        .get(&target_actor_id)
                        .map(|member| member.status),
                    Some(MemberStatus::Left | MemberStatus::Banned)
                ) {
                    community.subscribers.remove(&target_actor_id);
                }
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::MemberChanged,
                    Some(target_actor_id.clone()),
                    Some(format!("member:{}", target_actor_id.0)),
                    None,
                    joined_at_ms,
                );
                let participant_event = self
                    .state
                    .conversations
                    .get(&conversation_id)
                    .is_some_and(|conversation| {
                        matches!(
                            conversation.kind,
                            ConversationKind::Group | ConversationKind::Channel
                        )
                    })
                    .then(|| {
                        community
                            .members
                            .get(&target_actor_id)
                            .and_then(participant_for_community_member)
                            .map(|participant| Event::ConversationParticipantUpserted {
                                conversation_id: conversation_id.clone(),
                                participant,
                            })
                            .unwrap_or_else(|| Event::ConversationParticipantRemoved {
                                conversation_id: conversation_id.clone(),
                                actor_id: target_actor_id.clone(),
                            })
                    });
                let mut events = vec![Event::CommunityChanged { community }];
                if let Some(event) = participant_event {
                    events.push(event);
                }
                Ok(events)
            }
            Command::CreateInviteLink { actor_id, invite } => {
                let mut community = self
                    .state
                    .communities
                    .get(&invite.conversation_id)
                    .cloned()
                    .ok_or_else(|| {
                        EngineError::CommunityNotFound(invite.conversation_id.clone())
                    })?;
                require_community_admin(
                    &community,
                    &actor_id,
                    CommunityAdminAction::InviteMembers,
                )?;
                if invite.creator_id != actor_id
                    || invite.id.trim().is_empty()
                    || invite.token.trim().is_empty()
                    || invite.revoked
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let created_at_ms = invite.created_at_ms;
                let invite_id = invite.id.clone();
                community.invite_links.insert(invite_id.clone(), invite);
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::InviteCreated,
                    None,
                    Some(format!("invite:{}", invite_id)),
                    None,
                    created_at_ms,
                );
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::RevokeInviteLink {
                actor_id,
                conversation_id,
                invite_id,
                revoked_at_ms,
            } => {
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                require_community_admin(
                    &community,
                    &actor_id,
                    CommunityAdminAction::InviteMembers,
                )?;
                community.revoke_invite(&invite_id)?;
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::InviteRevoked,
                    None,
                    Some(format!("invite:{invite_id}")),
                    None,
                    revoked_at_ms,
                );
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::RequestCommunityJoin { actor_id, request } => {
                if request.actor_id != actor_id {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                self.require_actor(&actor_id)?;
                let conversation = self.require_conversation(&request.conversation_id)?;
                if !matches!(
                    conversation.kind,
                    ConversationKind::Group | ConversationKind::Channel
                ) {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let requested_actor_id = request.actor_id.clone();
                let requested_at_ms = request.requested_at_ms;
                let mut community = self
                    .state
                    .communities
                    .get(&request.conversation_id)
                    .cloned()
                    .ok_or_else(|| {
                        EngineError::CommunityNotFound(request.conversation_id.clone())
                    })?;
                if community
                    .members
                    .get(&actor_id)
                    .is_some_and(|member| matches!(member.status, MemberStatus::Banned))
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                if community
                    .public_username
                    .as_deref()
                    .is_none_or(|username| username.trim().is_empty())
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                community.request_join(request);
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::MemberChanged,
                    Some(requested_actor_id),
                    Some("join-request".into()),
                    None,
                    requested_at_ms,
                );
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::RespondCommunityJoin {
                actor_id,
                conversation_id,
                requester_id,
                approved,
                decided_at_ms,
            } => {
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                require_community_admin(
                    &community,
                    &actor_id,
                    CommunityAdminAction::InviteMembers,
                )?;
                if approved {
                    community.approve_join(&requester_id, &actor_id, decided_at_ms)?;
                    append_community_audit(
                        &mut community,
                        &actor_id,
                        CommunityAuditAction::JoinApproved,
                        Some(requester_id.clone()),
                        None,
                        None,
                        decided_at_ms,
                    );
                } else {
                    community
                        .pending_join_requests
                        .remove(&requester_id)
                        .ok_or_else(|| CommunityError::JoinRequestNotFound(requester_id.clone()))?;
                    append_community_audit(
                        &mut community,
                        &actor_id,
                        CommunityAuditAction::JoinRejected,
                        Some(requester_id.clone()),
                        None,
                        None,
                        decided_at_ms,
                    );
                }
                let participant_event = if approved
                    && self
                        .state
                        .conversations
                        .get(&conversation_id)
                        .is_some_and(|conversation| {
                            matches!(
                                conversation.kind,
                                ConversationKind::Group | ConversationKind::Channel
                            )
                        }) {
                    community
                        .members
                        .get(&requester_id)
                        .and_then(participant_for_community_member)
                        .map(|participant| Event::ConversationParticipantUpserted {
                            conversation_id: conversation_id.clone(),
                            participant,
                        })
                } else {
                    None
                };
                let mut events = vec![Event::CommunityChanged { community }];
                if let Some(event) = participant_event {
                    events.push(event);
                }
                Ok(events)
            }
            Command::UpsertForumTopic { actor_id, topic } => {
                let mut community = self
                    .state
                    .communities
                    .get(&topic.conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(topic.conversation_id.clone()))?;
                require_community_admin(&community, &actor_id, CommunityAdminAction::ManageTopics)?;
                if topic.id.trim().is_empty()
                    || topic.title.trim().is_empty()
                    || topic.title.trim().len() > 128
                    || topic.conversation_id != community.conversation_id
                {
                    return Err(EngineError::CommunityPermissionDenied);
                }
                let topic_id = topic.id.clone();
                let created_at_ms = topic.created_at_ms;
                community.topics.insert(topic_id.clone(), topic);
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::TopicUpserted,
                    None,
                    Some(format!("topic:{}", topic_id)),
                    None,
                    created_at_ms,
                );
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::DeleteForumTopic {
                actor_id,
                conversation_id,
                topic_id,
            } => {
                let mut community = self
                    .state
                    .communities
                    .get(&conversation_id)
                    .cloned()
                    .ok_or_else(|| EngineError::CommunityNotFound(conversation_id.clone()))?;
                require_community_admin(&community, &actor_id, CommunityAdminAction::ManageTopics)?;
                community.topics.remove(&topic_id);
                append_community_audit(
                    &mut community,
                    &actor_id,
                    CommunityAuditAction::TopicDeleted,
                    None,
                    Some(format!("topic:{}", topic_id)),
                    None,
                    0,
                );
                Ok(vec![Event::CommunityChanged { community }])
            }
            Command::RegisterBot { actor_id, profile } => {
                let actor = self.require_actor(&actor_id)?;
                if profile.actor_id != actor_id
                    || !matches!(actor.kind, ActorKind::Bot | ActorKind::Assistant)
                {
                    return Err(EngineError::BotPermissionDenied);
                }
                let mut registry = self.state.bots.clone();
                registry
                    .register(profile.clone())
                    .map_err(|error| EngineError::Bot(error.to_string()))?;
                Ok(vec![Event::BotRegistryChanged {
                    registry,
                    profile: Some(profile),
                    execution: None,
                }])
            }
            Command::BeginBotInvocation {
                actor_id,
                invocation,
                created_at_ms,
            } => {
                if invocation.sender_id != actor_id {
                    return Err(EngineError::BotPermissionDenied);
                }
                self.require_actor(&actor_id)?;
                self.require_conversation(&invocation.conversation_id)?;
                let mut registry = self.state.bots.clone();
                let execution = registry
                    .begin_execution(&invocation, created_at_ms)
                    .map_err(|error| EngineError::Bot(error.to_string()))?;
                let profile = registry.bots.get(&execution.bot_id).cloned();
                Ok(vec![Event::BotRegistryChanged {
                    registry,
                    profile,
                    execution: Some(execution),
                }])
            }
            Command::FinishBotExecution {
                actor_id,
                execution_id,
                success,
                finished_at_ms,
                error,
            } => {
                let mut registry = self.state.bots.clone();
                let execution =
                    registry
                        .executions
                        .get(&execution_id)
                        .cloned()
                        .ok_or_else(|| {
                            EngineError::Bot(format!("execution not found: {execution_id}"))
                        })?;
                if execution.bot_id != actor_id {
                    return Err(EngineError::BotPermissionDenied);
                }
                registry
                    .finish_execution(&execution_id, success, finished_at_ms, error)
                    .map_err(|error| EngineError::Bot(error.to_string()))?;
                let execution = registry.executions.get(&execution_id).cloned();
                let profile = execution
                    .as_ref()
                    .and_then(|execution| registry.bots.get(&execution.bot_id))
                    .cloned();
                Ok(vec![Event::BotRegistryChanged {
                    registry,
                    profile,
                    execution,
                }])
            }
            Command::InstallMiniApp { manifest } => Ok(vec![Event::MiniAppInstalled { manifest }]),
            Command::GrantMiniApp { grant } => {
                if !self.state.mini_apps.contains_key(&grant.mini_app_id) {
                    return Err(EngineError::MiniAppNotFound(grant.mini_app_id));
                }
                self.require_actor(&grant.actor_id)?;
                Ok(vec![Event::MiniAppGrantUpdated { grant }])
            }
            Command::OpenMiniApp { session } => {
                if !self.state.mini_apps.contains_key(&session.mini_app_id) {
                    return Err(EngineError::MiniAppNotFound(session.mini_app_id));
                }
                let grant = self
                    .state
                    .mini_app_grants
                    .get(&(session.mini_app_id.clone(), session.actor_id.clone()));
                if session.granted_permissions.iter().any(|permission| {
                    !grant.is_some_and(|grant| grant.permissions.contains(permission))
                }) {
                    return Err(EngineError::MiniAppPermissionDenied(
                        *session
                            .granted_permissions
                            .iter()
                            .find(|permission| {
                                !grant.is_some_and(|grant| grant.permissions.contains(permission))
                            })
                            .expect("permission exists"),
                    ));
                }
                Ok(vec![Event::MiniAppOpened { session }])
            }
            Command::MiniAppCall {
                session_id,
                request_id,
                request,
            } => {
                let session = self
                    .state
                    .mini_app_sessions
                    .get(&session_id)
                    .ok_or_else(|| EngineError::MiniAppSessionNotFound(session_id.clone()))?;
                if let Some(permission) = request.required_permission() {
                    if !session.granted_permissions.contains(&permission) {
                        return Err(EngineError::MiniAppPermissionDenied(permission));
                    }
                }
                let response = match request {
                    MiniAppRequest::Ready
                    | MiniAppRequest::Expand
                    | MiniAppRequest::Close
                    | MiniAppRequest::SetHeaderColor { .. }
                    | MiniAppRequest::SetBackgroundColor { .. } => MiniAppResponse::Ok,
                    _ => return Err(EngineError::MiniAppHostActionRequired),
                };
                Ok(vec![Event::MiniAppResponded {
                    session_id,
                    request_id,
                    response,
                }])
            }
        }
    }

    pub fn apply(&mut self, event: Event) {
        match event {
            Event::ActorUpserted { actor } => {
                self.state.actors.insert(actor.id.clone(), actor);
            }
            Event::PresenceUpdated { actor_id, presence } => {
                if let Some(actor) = self.state.actors.get_mut(&actor_id) {
                    actor.presence = presence;
                }
            }
            Event::ConversationUpserted { conversation } => {
                self.state
                    .conversations
                    .insert(conversation.id.clone(), conversation);
            }
            Event::ConversationInfoUpdated {
                conversation_id,
                title,
                description,
            } => {
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation.title = title;
                    conversation.description = description;
                }
            }
            Event::ConversationParticipantUpserted {
                conversation_id,
                participant,
            } => {
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    if let Some(existing) = conversation
                        .participants
                        .iter_mut()
                        .find(|item| item.actor_id == participant.actor_id)
                    {
                        *existing = participant;
                    } else {
                        conversation.participants.push(participant);
                    }
                }
            }
            Event::ConversationParticipantRemoved {
                conversation_id,
                actor_id,
            } => {
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation
                        .participants
                        .retain(|participant| participant.actor_id != actor_id);
                }
                if let Some(cursors) = self.state.read_cursors.get_mut(&conversation_id) {
                    cursors.remove(&actor_id);
                }
                if let Some(drafts) = self.state.drafts.get_mut(&conversation_id) {
                    drafts.remove(&actor_id);
                }
                if let Some(topic_cursors) = self.state.topic_read_cursors.get_mut(&conversation_id)
                {
                    topic_cursors.remove(&actor_id);
                }
                if let Some(topic_drafts) = self.state.topic_drafts.get_mut(&conversation_id) {
                    topic_drafts.remove(&actor_id);
                }
                if let Some(marked) = self.state.marked_unread_by_actor.get_mut(&conversation_id) {
                    marked.remove(&actor_id);
                }
            }
            Event::ConversationArchived {
                conversation_id,
                archived,
            } => {
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation.archived = archived;
                }
            }
            Event::ConversationPinned {
                conversation_id,
                pinned,
            } => {
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation.pinned = pinned;
                }
            }
            Event::ConversationMarkedUnread {
                conversation_id,
                actor_id,
                marked_unread,
            } => {
                let actors = self
                    .state
                    .marked_unread_by_actor
                    .entry(conversation_id.clone())
                    .or_default();
                if marked_unread {
                    actors.insert(actor_id.clone());
                } else {
                    actors.remove(&actor_id);
                }
                if actors.is_empty() {
                    self.state.marked_unread_by_actor.remove(&conversation_id);
                }
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation.marked_unread = marked_unread;
                }
            }
            Event::DraftChanged { draft } => {
                if draft.text.trim().is_empty() && draft.reply_to_message_id.is_none() {
                    if let Some(by_actor) = self.state.drafts.get_mut(&draft.conversation_id) {
                        by_actor.remove(&draft.actor_id);
                        if by_actor.is_empty() {
                            self.state.drafts.remove(&draft.conversation_id);
                        }
                    }
                } else {
                    self.state
                        .drafts
                        .entry(draft.conversation_id.clone())
                        .or_default()
                        .insert(draft.actor_id.clone(), draft);
                }
            }
            Event::ConversationNotificationsUpdated {
                conversation_id,
                settings,
            } => {
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation.notification_settings = settings;
                }
            }
            Event::FolderUpserted { folder } => {
                self.state.folders.insert(folder.id.clone(), folder);
            }
            Event::FolderDeleted { folder_id } => {
                self.state.folders.remove(&folder_id);
                for conversation in self.state.conversations.values_mut() {
                    conversation.folder_ids.retain(|id| id != &folder_id);
                }
            }
            Event::PresenceTriggeredSendQueued { pending } => {
                self.state
                    .pending_presence_sends
                    .insert(pending.client_message_id.clone(), pending);
            }
            Event::PresenceTriggeredSendRemoved { client_message_id } => {
                self.state.pending_presence_sends.remove(&client_message_id);
            }
            Event::MessageQueued { message } => {
                if let Some(conversation) =
                    self.state.conversations.get_mut(&message.conversation_id)
                {
                    conversation.last_message_id = Some(message.id.0.clone());
                    conversation.updated_at_ms = message.created_at_ms;
                }
                self.state
                    .messages
                    .entry(message.conversation_id.clone())
                    .or_default()
                    .insert(message.id.clone(), message.clone());
                if let Some(thread_root) = &message.thread_root_message_id {
                    if let Some(topic_id) = topic_id_from_root(thread_root) {
                        if let Some(topic) = self
                            .state
                            .communities
                            .get_mut(&message.conversation_id)
                            .and_then(|community| community.topics.get_mut(topic_id))
                        {
                            topic.last_message_id = Some(message.id.0);
                        }
                    }
                }
            }
            Event::MessageAcknowledged {
                conversation_id,
                local_message_id,
                server_message_id,
                accepted_at_ms,
            } => {
                if let Some(messages) = self.state.messages.get_mut(&conversation_id) {
                    if let Some(mut message) = messages.remove(&local_message_id) {
                        message.id = server_message_id.clone();
                        message.delivery_state = DeliveryState::Sent;
                        message.created_at_ms = accepted_at_ms;
                        messages.insert(server_message_id.clone(), message);
                    }
                }
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    if conversation.last_message_id.as_deref() == Some(local_message_id.0.as_str())
                    {
                        conversation.last_message_id = Some(server_message_id.0);
                    }
                }
            }
            Event::DeliveryStateUpdated {
                conversation_id,
                message_id,
                state,
            } => {
                if let Some(message) = self
                    .state
                    .messages
                    .get_mut(&conversation_id)
                    .and_then(|messages| messages.get_mut(&message_id))
                {
                    message.delivery_state = state;
                }
            }
            Event::MessageEdited {
                conversation_id,
                message_id,
                content,
                edited_at_ms,
            } => {
                if let Some(message) = self
                    .state
                    .messages
                    .get_mut(&conversation_id)
                    .and_then(|messages| messages.get_mut(&message_id))
                {
                    message.content = content;
                    message.edited_at_ms = Some(edited_at_ms);
                }
            }
            Event::MessagesDeleted {
                conversation_id,
                message_ids,
            } => {
                for id in message_ids {
                    let message = self
                        .state
                        .messages
                        .get(&conversation_id)
                        .and_then(|messages| messages.get(&id))
                        .cloned();
                    if let Some(message) = message.as_ref() {
                        for child in &mut self.state.conversation_child_states {
                            let exact_member = match child.destination.child.as_ref() {
                                Some(ConversationChildIdentity::Topic { root_message_id }) => {
                                    child.destination.conversation_id == conversation_id
                                        && message
                                            .thread_root_message_id
                                            .as_ref()
                                            .is_some_and(|root| &root.0 == root_message_id)
                                }
                                Some(ConversationChildIdentity::SavedSublist { .. }) => {
                                    child.destination.conversation_id == conversation_id
                                        && child.has_authoritative_message(&id.0)
                                }
                                Some(ConversationChildIdentity::Conversation {
                                    conversation_id: child_conversation_id,
                                }) => child_conversation_id == &conversation_id,
                                None => false,
                            };
                            if exact_member {
                                child.remove_message(&id.0);
                            }
                        }
                    }
                    if let Some(message) = self
                        .state
                        .messages
                        .get_mut(&conversation_id)
                        .and_then(|messages| messages.get_mut(&id))
                    {
                        message.deleted = true;
                    }
                    if let Some(conversation) =
                        self.state.conversations.get_mut(&conversation_id)
                    {
                        conversation.pinned_message_ids.retain(|pinned| pinned != &id.0);
                    }
                    let remove_poll_bucket = self
                        .state
                        .poll_votes
                        .get_mut(&conversation_id)
                        .is_some_and(|by_message| {
                            by_message.remove(&id);
                            by_message.is_empty()
                        });
                    if remove_poll_bucket {
                        self.state.poll_votes.remove(&conversation_id);
                    }
                }
            }
            Event::ConversationRead {
                conversation_id,
                actor_id,
                message_id,
            } => {
                self.state
                    .read_cursors
                    .entry(conversation_id.clone())
                    .or_default()
                    .insert(actor_id.clone(), message_id.clone());
                if let Some(actors) = self.state.marked_unread_by_actor.get_mut(&conversation_id) {
                    actors.remove(&actor_id);
                    if actors.is_empty() {
                        self.state.marked_unread_by_actor.remove(&conversation_id);
                    }
                }
                // Keep legacy snapshot fields coherent for older readers. Actor-specific
                // clients receive the authoritative read state from the projected sync view.
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation.last_read_message_id = Some(message_id.0);
                    conversation.unread_count = 0;
                    conversation.marked_unread = false;
                }
            }
            Event::TopicReadChanged {
                conversation_id,
                topic_id,
                actor_id,
                message_id,
            } => {
                let position = self
                    .state
                    .messages
                    .get(&conversation_id)
                    .and_then(|messages| messages.get(&message_id))
                    .map(|message| {
                        ConversationMessagePosition::new(
                            message.created_at_ms,
                            message.id.0.clone(),
                        )
                    });
                if let Some(position) = position {
                    let destination = ConversationDestination::topic(
                        conversation_id.clone(),
                        format!("topic:{topic_id}"),
                    );
                    if let Some(child) =
                        self.state.child_state_mut(destination, actor_id.clone())
                    {
                        let _ = child.advance_inbox_read_till(position, None);
                    }
                }
                // Protocol compatibility projection while topic callers migrate to
                // the source-neutral child runtime state above.
                self.state
                    .topic_read_cursors
                    .entry(conversation_id)
                    .or_default()
                    .entry(actor_id)
                    .or_default()
                    .insert(topic_id, message_id);
            }
            Event::ConversationChildReadChanged {
                destination,
                actor_id,
                message_id,
            } => {
                let position = self
                    .state
                    .messages
                    .get(destination_message_conversation_id(&destination))
                    .and_then(|messages| messages.get(&message_id))
                    .map(|message| {
                        ConversationMessagePosition::new(
                            message.created_at_ms,
                            message.id.0.clone(),
                        )
                    });
                if let Some(position) = position {
                    let pending = self
                        .state
                        .conversation_child_states
                        .iter()
                        .find(|child| {
                            child.destination == destination && child.actor_id == actor_id
                        })
                        .map(|child| child.pending_incoming_notification_message_ids.clone())
                        .unwrap_or_default();
                    let message_conversation_id =
                        destination_message_conversation_id(&destination).clone();
                    let clear_ids = self
                        .state
                        .messages
                        .get(&message_conversation_id)
                        .map(|messages| {
                            pending
                                .iter()
                                .filter_map(|id| {
                                    let message_id = MessageId(id.clone());
                                    let message = messages.get(&message_id)?;
                                    let candidate = ConversationMessagePosition::new(
                                        message.created_at_ms,
                                        message.id.0.clone(),
                                    );
                                    (candidate <= position).then(|| id.clone())
                                })
                                .collect::<BTreeSet<_>>()
                        })
                        .unwrap_or_default();
                    if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                        if child.advance_inbox_read_till(position, None) {
                            child
                                .pending_incoming_notification_message_ids
                                .retain(|id| !clear_ids.contains(id));
                        }
                    }
                }
            }
            Event::TopicDraftChanged { draft } => {
                let destination = ConversationDestination::topic(
                    draft.conversation_id.clone(),
                    format!("topic:{}", draft.topic_id),
                );
                if let Some(child) = self
                    .state
                    .child_state_mut(destination, draft.actor_id.clone())
                {
                    if draft.text.trim().is_empty() && draft.reply_to_message_id.is_none() {
                        child.clear_draft();
                    } else {
                        child.set_draft(
                            draft.text.clone(),
                            draft.reply_to_message_id.clone(),
                            draft.updated_at_ms,
                        );
                    }
                }
                if draft.text.trim().is_empty() && draft.reply_to_message_id.is_none() {
                    if let Some(by_actor) = self.state.topic_drafts.get_mut(&draft.conversation_id)
                    {
                        if let Some(by_topic) = by_actor.get_mut(&draft.actor_id) {
                            by_topic.remove(&draft.topic_id);
                            if by_topic.is_empty() {
                                by_actor.remove(&draft.actor_id);
                            }
                        }
                        if by_actor.is_empty() {
                            self.state.topic_drafts.remove(&draft.conversation_id);
                        }
                    }
                } else {
                    self.state
                        .topic_drafts
                        .entry(draft.conversation_id.clone())
                        .or_default()
                        .entry(draft.actor_id.clone())
                        .or_default()
                        .insert(draft.topic_id.clone(), draft);
                }
            }
            Event::ConversationChildDraftChanged {
                destination,
                actor_id,
                text,
                reply_to_message_id,
                updated_at_ms,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    if text.trim().is_empty() && reply_to_message_id.is_none() {
                        child.clear_draft();
                    } else {
                        child.set_draft(text, reply_to_message_id, updated_at_ms);
                    }
                }
            }
            Event::ConversationChildWindowReplaced {
                destination,
                actor_id,
                message_ids,
                skipped_before,
                skipped_after,
                full_count,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    let empty = message_ids.is_empty();
                    if child.pagination.replace_window(
                        message_ids,
                        skipped_before,
                        skipped_after,
                        full_count,
                    ) {
                        if empty {
                            child.note_locally_empty();
                        } else {
                            child.note_non_empty();
                        }
                    }
                }
            }
            Event::ConversationChildPinnedChanged {
                destination,
                actor_id,
                pinned,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    child.pinned = pinned;
                    if pinned {
                        child.restore_pinned_when_non_empty = false;
                    }
                }
            }
            Event::ConversationChildActiveChanged {
                destination,
                actor_id,
                active,
            } => {
                if active {
                    for child in &mut self.state.conversation_child_states {
                        if child.actor_id == actor_id
                            && child.destination.conversation_id == destination.conversation_id
                        {
                            child.active = false;
                        }
                    }
                    self.note_destination_opened(&actor_id, &destination);
                }
                if let Some(child) = self
                    .state
                    .child_state_mut(destination.clone(), actor_id.clone())
                {
                    child.set_active(active);
                }
            }
            Event::ConversationChildMarkedUnreadChanged {
                destination,
                actor_id,
                marked_unread,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    child.marked_unread = marked_unread;
                }
            }
            Event::ConversationChildUnreadThingsReconciled {
                destination,
                actor_id,
                unread_things,
                pending_incoming_notification_message_ids,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    child.unread_things = unread_things;
                    child.replace_pending_incoming_notifications(
                        pending_incoming_notification_message_ids,
                    );
                }
            }
            Event::SavedSublistMembershipReconciled {
                destination,
                actor_id,
                message_ids,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    let accepted = child.reconcile_authoritative_message_ids(message_ids);
                    debug_assert!(accepted, "validated SavedSublist membership event");
                }
            }
            Event::SavedSublistParentAccessReconciled {
                conversation_id,
                actor_id,
                allowed,
            } => {
                if allowed {
                    self.state
                        .saved_sublist_parent_access
                        .entry(conversation_id)
                        .or_default()
                        .insert(actor_id);
                } else {
                    let remove_parent = self
                        .state
                        .saved_sublist_parent_access
                        .get_mut(&conversation_id)
                        .is_some_and(|actors| {
                            actors.remove(&actor_id);
                            actors.is_empty()
                        });
                    if remove_parent {
                        self.state.saved_sublist_parent_access.remove(&conversation_id);
                    }
                    self.state.conversation_child_states.retain(|child| {
                        !(child.actor_id == actor_id
                            && child.destination.conversation_id == conversation_id
                            && matches!(
                                child.destination.child,
                                Some(ConversationChildIdentity::SavedSublist { .. })
                            ))
                    });
                    let remove_actor = self
                        .recent_open_destinations
                        .get_mut(&actor_id)
                        .is_some_and(|recent| {
                            recent.retain(|destination| {
                                !(destination.conversation_id == conversation_id
                                    && matches!(
                                        &destination.child,
                                        Some(ConversationChildIdentity::SavedSublist { .. })
                                    ))
                            });
                            recent.is_empty()
                        });
                    if remove_actor {
                        self.recent_open_destinations.remove(&actor_id);
                    }
                }
            }
            Event::ConversationChildNoPaidMessagesChanged {
                destination,
                actor_id,
                no_paid_messages,
            } => {
                if let Some(child) = self.state.child_state_mut(destination, actor_id) {
                    child.no_paid_messages = no_paid_messages;
                }
            }
            Event::ConversationChildDestroyed {
                destination,
                actor_id,
            } => {
                self.state.conversation_child_states.retain(|child| {
                    child.destination != destination || child.actor_id != actor_id
                });
                self.remove_recent_destination(&actor_id, &destination);
            }
            Event::ReactionUpdated {
                conversation_id,
                message_id,
                reaction,
            } => {
                if let Some(message) = self
                    .state
                    .messages
                    .get_mut(&conversation_id)
                    .and_then(|messages| messages.get_mut(&message_id))
                {
                    message
                        .reactions
                        .retain(|item| item.reaction != reaction.reaction);
                    if reaction.count > 0 {
                        message.reactions.push(reaction);
                    }
                }
            }
            Event::MessagePinned {
                conversation_id,
                message_id,
                pinned,
            } => {
                if let Some(message) = self
                    .state
                    .messages
                    .get_mut(&conversation_id)
                    .and_then(|messages| messages.get_mut(&message_id))
                {
                    message.pinned = pinned;
                }
                if let Some(conversation) = self.state.conversations.get_mut(&conversation_id) {
                    conversation
                        .pinned_message_ids
                        .retain(|id| id != &message_id.0);
                    if pinned {
                        conversation.pinned_message_ids.push(message_id.0);
                    }
                }
            }
            Event::PollVoteChanged {
                conversation_id,
                message_id,
                actor_id,
                option_ids,
            } => {
                let selected = option_ids.into_iter().collect::<BTreeSet<_>>();
                let by_message = self
                    .state
                    .poll_votes
                    .entry(conversation_id.clone())
                    .or_default();
                let by_actor = by_message.entry(message_id.clone()).or_default();
                if selected.is_empty() {
                    by_actor.remove(&actor_id);
                } else {
                    by_actor.insert(actor_id, selected);
                }
                if let Some(message) = self
                    .state
                    .messages
                    .get_mut(&conversation_id)
                    .and_then(|messages| messages.get_mut(&message_id))
                {
                    if let MessageContent::Poll { options, .. } = &mut message.content {
                        for option in options {
                            option.voter_count = u32::try_from(
                                by_actor
                                    .values()
                                    .filter(|votes| votes.contains(&option.id))
                                    .count(),
                            )
                            .unwrap_or(u32::MAX);
                            option.chosen = false;
                        }
                    }
                }
            }
            Event::InvoiceCreated { invoice } => {
                self.state.invoices.insert(invoice.id.clone(), invoice);
            }
            Event::OrderUpserted { order } => {
                self.state.orders.insert(order.id.clone(), order);
            }
            Event::WalletChanged { wallet, .. } => {
                self.state.wallet = wallet;
            }
            Event::WalletRuntimeChanged { runtime } => {
                self.state.wallet.runtime = runtime;
            }
            Event::ConnectedAppStateChanged { state } => {
                self.state.connected_apps = state;
            }
            Event::EntitlementReconciled { entitlement } => {
                self.state.entitlements.insert(entitlement.id.clone(), entitlement);
            }
            Event::StoryStealthChanged { actor_id, state } => {
                self.state.story_stealth.insert(actor_id, state);
            }
            Event::StoryChanged { story } => {
                self.state.stories.insert(story.id.clone(), story);
            }
            Event::StoryDeleted { story_id } => {
                self.state.stories.remove(&story_id);
            }
            Event::CommunityChanged { community } => {
                self.state
                    .communities
                    .insert(community.conversation_id.clone(), community);
            }
            Event::BotRegistryChanged { registry, .. } => {
                self.state.bots = registry;
            }
            Event::MiniAppInstalled { manifest } => {
                self.state.mini_apps.insert(manifest.id.clone(), manifest);
            }
            Event::MiniAppGrantUpdated { grant } => {
                self.state
                    .mini_app_grants
                    .insert((grant.mini_app_id.clone(), grant.actor_id.clone()), grant);
            }
            Event::MiniAppOpened { session } => {
                self.state
                    .mini_app_sessions
                    .insert(session.id.clone(), session);
            }
            Event::MiniAppResponded { .. } => {}
        }
    }

    pub fn complete_paid_order(
        &mut self,
        order_id: &str,
        updated_at_ms: i64,
    ) -> Result<Vec<Event>, EngineError> {
        let mut order = self
            .state
            .orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| EngineError::OrderNotFound(order_id.to_string()))?;
        order.status = PaymentStatus::Paid;
        order.updated_at_ms = updated_at_ms;
        self.execute(Command::UpsertOrder { order })
    }

    fn require_actor(&self, id: &ActorId) -> Result<&Actor, EngineError> {
        self.state
            .actors
            .get(id)
            .ok_or_else(|| EngineError::ActorNotFound(id.clone()))
    }
    fn require_conversation(&self, id: &ConversationId) -> Result<&Conversation, EngineError> {
        self.state
            .conversations
            .get(id)
            .ok_or_else(|| EngineError::ConversationNotFound(id.clone()))
    }
    fn require_message(
        &self,
        conversation_id: &ConversationId,
        message_id: &MessageId,
    ) -> Result<&Message, EngineError> {
        self.state
            .messages
            .get(conversation_id)
            .and_then(|messages| messages.get(message_id))
            .ok_or_else(|| EngineError::MessageNotFound {
                conversation_id: conversation_id.clone(),
                message_id: message_id.clone(),
            })
    }
}


#[cfg(test)]
mod recent_open_history_tests {
    use super::*;

    #[test]
    fn recent_open_history_is_bounded_deduped_and_move_front() {
        let actor_id = ActorId::new("human:recent-open");
        let parent_id = ConversationId::new("conversation:recent-open-parent");
        let mut engine = MessagingEngine::new();

        for index in 0..=MAX_RECENT_OPEN_DESTINATIONS {
            let destination = ConversationDestination::nested_conversation(
                parent_id.clone(),
                ConversationId::new(format!("conversation:recent-open-child:{index}")),
            );
            engine.note_destination_opened(&actor_id, &destination);
        }

        let recent = engine.recent_open_destinations(&actor_id);
        assert_eq!(recent.len(), MAX_RECENT_OPEN_DESTINATIONS);
        assert_eq!(
            recent.first(),
            Some(&ConversationDestination::nested_conversation(
                parent_id.clone(),
                ConversationId::new(format!(
                    "conversation:recent-open-child:{}",
                    MAX_RECENT_OPEN_DESTINATIONS
                )),
            ))
        );
        assert!(!recent.contains(&ConversationDestination::nested_conversation(
            parent_id.clone(),
            ConversationId::new("conversation:recent-open-child:0"),
        )));

        let existing = ConversationDestination::nested_conversation(
            parent_id,
            ConversationId::new("conversation:recent-open-child:5"),
        );
        engine.note_destination_opened(&actor_id, &existing);
        let recent = engine.recent_open_destinations(&actor_id);
        assert_eq!(recent.len(), MAX_RECENT_OPEN_DESTINATIONS);
        assert_eq!(recent.first(), Some(&existing));
        assert_eq!(recent.iter().filter(|item| *item == &existing).count(), 1);
    }

    #[test]
    fn deleted_message_cleans_only_authoritative_saved_sublist_membership() {
        let parent = ConversationId::new("conversation:self");
        let actor_id = ActorId::new("human:self");
        let message_id = MessageId::new("message:delete");
        let mut owned = ConversationChildRuntimeState::new(
            ConversationDestination::saved_sublist(
                parent.clone(),
                ActorId::new("human:owned"),
            ),
            actor_id.clone(),
        )
        .expect("valid owned child");
        let mut unrelated = ConversationChildRuntimeState::new(
            ConversationDestination::saved_sublist(
                parent.clone(),
                ActorId::new("human:unrelated"),
            ),
            actor_id.clone(),
        )
        .expect("valid unrelated child");

        assert!(owned.reconcile_authoritative_message_ids(vec![message_id.0.clone()]));
        assert!(unrelated.reconcile_authoritative_message_ids(vec![
            "message:other".into()
        ]));
        for child in [&mut owned, &mut unrelated] {
            assert!(child.pagination.replace_window(
                child.authoritative_message_ids.clone(),
                Some(0),
                Some(0),
                Some(1),
            ));
            child.replace_pending_incoming_notifications(
                child.authoritative_message_ids.clone(),
            );
            child.unread_count = Some(1);
        }

        let message = Message {
            id: message_id.clone(),
            conversation_id: parent.clone(),
            sender_id: ActorId::new("human:peer"),
            content: MessageContent::Text {
                text: crate::message::FormattedText::plain("delete me"),
            },
            reply_to_message_id: None,
            thread_root_message_id: None,
            forward_origin: None,
            reply_markup: None,
            reactions: Vec::new(),
            delivery_state: DeliveryState::Delivered,
            created_at_ms: 10,
            edited_at_ms: None,
            scheduled_at_ms: None,
            silent: false,
            protected_content: false,
            pinned: false,
            deleted: false,
        };

        let mut state = MessagingState::default();
        state
            .messages
            .entry(parent.clone())
            .or_default()
            .insert(message_id.clone(), message);
        state.conversation_child_states = vec![owned, unrelated];
        let mut engine = MessagingEngine::from_state(state);

        engine.apply(Event::MessagesDeleted {
            conversation_id: parent.clone(),
            message_ids: vec![message_id.clone()],
        });

        let owned = &engine.state().conversation_child_states[0];
        assert!(owned.authoritative_message_ids.is_empty());
        assert!(owned.pagination.message_ids.is_empty());
        assert!(owned.pending_incoming_notification_message_ids.is_empty());
        assert!(owned.unread_count.is_none());

        let unrelated = &engine.state().conversation_child_states[1];
        assert_eq!(unrelated.authoritative_message_ids, vec!["message:other"]);
        assert_eq!(unrelated.pagination.message_ids, vec!["message:other"]);
        assert_eq!(
            unrelated.pending_incoming_notification_message_ids,
            vec!["message:other"]
        );
        assert_eq!(unrelated.unread_count, Some(1));
        assert!(
            engine
                .state()
                .messages
                .get(&parent)
                .and_then(|messages| messages.get(&message_id))
                .is_some_and(|message| message.deleted)
        );
    }

    #[test]
    fn recent_open_history_is_runtime_only_across_state_restore() {
        let actor_id = ActorId::new("human:recent-open");
        let destination = ConversationDestination::nested_conversation(
            ConversationId::new("conversation:recent-open-parent"),
            ConversationId::new("conversation:recent-open-child"),
        );
        let mut engine = MessagingEngine::new();
        engine.note_destination_opened(&actor_id, &destination);
        assert_eq!(engine.recent_open_destinations(&actor_id), &[destination]);

        let restored = MessagingEngine::from_state(engine.state().clone());
        assert!(restored.recent_open_destinations(&actor_id).is_empty());
    }
}


#[cfg(test)]
mod wallet_runtime_command_tests {
    use super::*;

    #[test]
    fn wallet_rate_runtime_commands_are_durable_and_fail_closed() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::SetWalletFiatCurrency {
                currency: "eur".into(),
            })
            .unwrap();
        engine
            .execute(Command::ApplyWalletRateSnapshot {
                rates: BTreeMap::from([
                    ("EUR".into(), 920_000),
                    ("USD".into(), 1_000_000),
                ]),
                observed_at_ms: 1_000,
            })
            .unwrap();
        assert_eq!(
            engine.state().wallet.runtime.rates.current_quote_micros(),
            Some(920_000)
        );
        assert!(matches!(
            engine.execute(Command::SetWalletFiatCurrency {
                currency: "not-a-currency".into(),
            }),
            Err(EngineError::WalletRate(WalletRateError::InvalidCurrency(_)))
        ));
    }

    #[test]
    fn wallet_funding_runtime_fences_replaced_request_generation() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::BeginWalletFunding {
                address: "first-address".into(),
                asset: "asset".into(),
                base_currency: Some("EUR".into()),
            })
            .unwrap();
        let first = engine
            .state()
            .wallet
            .runtime
            .onramp
            .active
            .as_ref()
            .unwrap()
            .request_id;
        engine
            .execute(Command::BeginWalletFunding {
                address: "second-address".into(),
                asset: "asset".into(),
                base_currency: Some("EUR".into()),
            })
            .unwrap();
        let second = engine
            .state()
            .wallet
            .runtime
            .onramp
            .active
            .as_ref()
            .unwrap()
            .request_id;

        assert!(matches!(
            engine.execute(Command::ResolveWalletFundingProvider {
                request_id: first,
                providers: vec![OnrampProviderInfo::new(
                    "provider-old",
                    None::<Vec<String>>,
                )],
            }),
            Err(EngineError::WalletOnramp(WalletOnrampError::StaleRequest {
                expected,
                received
            })) if expected == second && received == first
        ));

        engine
            .execute(Command::ResolveWalletFundingProvider {
                request_id: second,
                providers: vec![OnrampProviderInfo::new(
                    "provider-current",
                    Some(vec!["USD".to_string()]),
                )],
            })
            .unwrap();
        engine
            .execute(Command::CompleteWalletFunding {
                request_id: second,
                session_url: Some("https://example.invalid/funding".into()),
            })
            .unwrap();
        let active = engine.state().wallet.runtime.onramp.active.as_ref().unwrap();
        assert_eq!(active.provider_id.as_deref(), Some("provider-current"));
        assert_eq!(
            active.session_url.as_deref(),
            Some("https://example.invalid/funding")
        );
    }

    #[test]
    fn wallet_panel_runtime_reuses_single_shipping_surface_generation() {
        let mut engine = MessagingEngine::new();
        engine.execute(Command::ShowWalletPanel).unwrap();
        let generation = engine.state().wallet.runtime.panel.generation;
        engine.execute(Command::MinimizeWalletPanel).unwrap();
        engine.execute(Command::ShowWalletPanel).unwrap();
        assert_eq!(engine.state().wallet.runtime.panel.generation, generation);
        engine
            .execute(Command::SetWalletTransactionsVisible { visible: true })
            .unwrap();
        assert!(engine.state().wallet.runtime.panel.transactions_visible);
        engine.execute(Command::CloseWalletPanel).unwrap();
        assert!(matches!(
            engine.execute(Command::CloseWalletPanel),
            Err(EngineError::WalletPanelNotVisible)
        ));
    }
}


#[cfg(test)]
mod connected_app_engine_tests {
    use super::*;
    use crate::connected_app::{
        ConnectedAppClaimDecision, ConnectedAppManifest, ConnectedAppRequest,
        ConnectedAppRequestKind, ConnectedAppSession, ConnectedAppSessionStatus,
    };

    fn connected_session() -> ConnectedAppSession {
        ConnectedAppSession {
            id: 91,
            client_id: "app-client".into(),
            manifest: Some(ConnectedAppManifest {
                url: "https://app.example/manifest.json".into(),
                name: "Connected App".into(),
                icon_url: None,
            }),
            status: ConnectedAppSessionStatus::Active,
            created_at_ms: 10,
            updated_at_ms: 10,
        }
    }

    #[test]
    fn connected_app_runtime_is_persisted_by_the_canonical_engine() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::UpsertConnectedAppSession {
                session: connected_session(),
            })
            .unwrap();
        engine
            .execute(Command::QueueConnectedAppRequest {
                request: ConnectedAppRequest {
                    session_id: 91,
                    request_id: "request-1".into(),
                    method: "sendTransaction".into(),
                    kind: ConnectedAppRequestKind::SendTransaction,
                    trace_id: "trace-1".into(),
                    expires_at_ms: 1_000,
                },
                observed_at_ms: 100,
            })
            .unwrap();
        engine
            .execute(Command::ResolveConnectedAppRequest {
                session_id: 91,
                request_id: "request-1".into(),
                decision: ConnectedAppClaimDecision::Confirm,
                operation_id: "operation-1".into(),
                signed_payload: "signed-payload".into(),
                answer: vec![1, 2, 3],
                observed_at_ms: 200,
            })
            .unwrap();

        assert!(engine.state().connected_apps.pending_requests.is_empty());
        assert_eq!(engine.state().connected_apps.claims.len(), 1);
        let encoded = serde_json::to_string(engine.state()).unwrap();
        let restored: MessagingState = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.connected_apps.claims.len(), 1);
    }

    #[test]
    fn connected_app_wallet_claim_recovery_is_bound_to_wallet_key() {
        let mut engine = MessagingEngine::new();
        engine.execute(Command::UpsertConnectedAppSession {
            session: connected_session(),
        }).unwrap();
        engine.execute(Command::QueueConnectedAppRequest {
            request: ConnectedAppRequest {
                session_id: 91,
                request_id: "wallet-request".into(),
                method: "sendTransaction".into(),
                kind: ConnectedAppRequestKind::SendTransaction,
                trace_id: "trace-wallet".into(),
                expires_at_ms: 1_000,
            },
            observed_at_ms: 100,
        }).unwrap();
        let wallet = WalletTransferIdentity {
            network: 1,
            address: "EQ-wallet".into(),
            public_key: vec![8; 32],
            revision: 3,
        };
        engine.execute(Command::ResolveConnectedAppWalletRequest {
            session_id: 91,
            request_id: "wallet-request".into(),
            decision: ConnectedAppClaimDecision::Confirm,
            wallet_identity: wallet.clone(),
            operation_id: "operation-wallet".into(),
            signed_payload: "signed-wallet".into(),
            answer: vec![9],
            observed_at_ms: 200,
        }).unwrap();
        let encoded = serde_json::to_string(engine.state()).unwrap();
        let restored: MessagingState = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            restored.connected_apps.recoverable_claims_for_wallet(&wallet, 500).len(),
            1
        );
        let another_key = WalletTransferIdentity {
            public_key: vec![7; 32],
            ..wallet
        };
        assert!(restored
            .connected_apps
            .recoverable_claims_for_wallet(&another_key, 500)
            .is_empty());
    }

    #[test]
    fn closing_connected_app_session_cancels_owned_pending_requests() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::UpsertConnectedAppSession {
                session: connected_session(),
            })
            .unwrap();
        engine
            .execute(Command::QueueConnectedAppRequest {
                request: ConnectedAppRequest {
                    session_id: 91,
                    request_id: "request-close".into(),
                    method: "signData".into(),
                    kind: ConnectedAppRequestKind::SignData,
                    trace_id: String::new(),
                    expires_at_ms: 1_000,
                },
                observed_at_ms: 100,
            })
            .unwrap();
        engine
            .execute(Command::CloseConnectedAppSession {
                session_id: 91,
                closed_at_ms: 200,
            })
            .unwrap();
        assert!(engine.state().connected_apps.pending_requests.is_empty());
        assert_eq!(
            engine.state().connected_apps.sessions.get(&91).unwrap().status,
            ConnectedAppSessionStatus::Closed
        );
    }
}


#[cfg(test)]
mod outbound_transfer_engine_tests {
    use super::*;
    use crate::wallet::{
        OutboundTransferHandoff, OutboundTransferRecord, OutboundTransferTerminal,
    };

    fn transfer() -> OutboundTransferRecord {
        OutboundTransferRecord {
            record_id: "transfer-1".into(),
            network: 1,
            address: "source-address".into(),
            public_key: vec![1; 32],
            operation_id: "operation-1".into(),
            destination: "destination-address".into(),
            comment: String::new(),
            collectible: None,
            amount_nano: 100_000_000,
            posted_at_ms: 100,
            handoff: OutboundTransferHandoff::Preparation,
            terminal: OutboundTransferTerminal::None,
            message_token: None,
            confirmed_hash: None,
            lookup_attempts: 0,
            lookup_stopped: false,
            paired: false,
            bounce: false,
        }
    }

    #[test]
    fn engine_persists_submission_unknown_before_lookup_or_terminal_settlement() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::JournalOutboundTransfer { record: transfer() })
            .unwrap();
        engine
            .execute(Command::MarkOutboundTransferHandoff {
                record_id: "transfer-1".into(),
                message_token: vec![8; 32],
            })
            .unwrap();
        assert_eq!(
            engine
                .state()
                .wallet
                .runtime
                .outbound_transfers
                .submission_unknown()
                .len(),
            1
        );

        engine
            .execute(Command::RecordOutboundTransferLookup {
                record_id: "transfer-1".into(),
            })
            .unwrap();
        engine
            .execute(Command::SettleOutboundTransfer {
                record_id: "transfer-1".into(),
                terminal: OutboundTransferTerminal::Confirmed,
                confirmed_hash: Some(vec![9; 32]),
            })
            .unwrap();
        assert!(engine
            .state()
            .wallet
            .runtime
            .outbound_transfers
            .submission_unknown()
            .is_empty());
    }
}


#[cfg(test)]
mod wallet_address_engine_tests {
    use super::*;
    use crate::actor::Actor;
    use crate::wallet::WalletAddressKnowledge;

    fn person(id: &str) -> Actor {
        Actor::human(id, id)
    }

    #[test]
    fn address_reconciliation_is_server_authoritative_and_stale_safe() {
        let mut engine = MessagingEngine::new();
        let actor = person("person-address");
        let actor_id = actor.id.clone();
        engine.execute(Command::UpsertActor { actor }).unwrap();
        engine
            .execute(Command::ReconcileWalletUserAddress {
                actor_id: actor_id.clone(),
                serial: 2,
                address: Some("EQ-current".into()),
                public_key: vec![4; 32],
            })
            .unwrap();
        engine
            .execute(Command::ReconcileWalletUserAddress {
                actor_id: actor_id.clone(),
                serial: 1,
                address: Some("EQ-stale".into()),
                public_key: vec![3; 32],
            })
            .unwrap();
        assert_eq!(
            engine.state().wallet.runtime.address_directory.known(&actor_id).address,
            "EQ-current"
        );
        engine
            .execute(Command::SetWalletAddressServiceUnavailable {
                unavailable: true,
            })
            .unwrap();
        assert_eq!(
            engine.state().wallet.runtime.address_directory.known(&ActorId("unknown".into())).knowledge,
            WalletAddressKnowledge::Unknown
        );
    }
}


#[cfg(test)]
mod wallet_quoted_transfer_engine_tests {
    use super::*;
    use crate::wallet::{OutboundTransferHandoff, WalletSponsoredFeeState};

    fn identity() -> WalletTransferIdentity {
        WalletTransferIdentity {
            network: 1,
            address: "EQ-wallet".into(),
            public_key: vec![6; 32],
            revision: 1,
        }
    }

    #[test]
    fn engine_journals_sponsored_quote_through_single_outbound_owner() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::ResetWalletSponsoredFeeGeneration {
                network_generation: 6,
            })
            .unwrap();
        let who = identity();
        engine
            .execute(Command::BeginWalletSponsoredFeeRequest {
                network_generation: 6,
                identity: who.clone(),
                transfer_min_nano: 100_000_000,
                configured_min_nano: 100_000_000,
                observed_at_ms: 1_000,
                force: false,
            })
            .unwrap();
        let serial = engine
            .state()
            .wallet
            .runtime
            .sponsored_fees
            .active_request
            .as_ref()
            .unwrap()
            .serial;
        engine
            .execute(Command::ApplyWalletSponsoredFeeInfo {
                serial,
                network_generation: 6,
                identity: who.clone(),
                info: WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 100_000_000,
                    reset_at_ms: 100_000,
                    left: 2,
                    available: true,
                },
                observed_at_ms: 2_000,
            })
            .unwrap();

        let mut fee_state: WalletSponsoredFeeState =
            engine.state().wallet.runtime.sponsored_fees.clone();
        let quote = fee_state
            .quote_transfer(
                6,
                who.clone(),
                100_000_000,
                100_000_000,
                300_000_000,
                "EQ-destination".into(),
                25_000_000,
                2_001,
            )
            .unwrap();
        assert!(quote.paired);
        let record = OutboundTransferRecord {
            record_id: "r1".into(),
            network: who.network,
            address: who.address.clone(),
            public_key: who.public_key.clone(),
            operation_id: "op1".into(),
            destination: quote.destination.clone(),
            comment: String::new(),
            collectible: None,
            amount_nano: quote.amount_nano,
            posted_at_ms: 2_001,
            handoff: OutboundTransferHandoff::Preparation,
            terminal: OutboundTransferTerminal::None,
            message_token: None,
            confirmed_hash: None,
            lookup_attempts: 0,
            lookup_stopped: false,
            paired: true,
            bounce: false,
        };
        engine
            .execute(Command::JournalQuotedOutboundTransfer {
                record,
                quote,
                balance_nano: 300_000_000,
                observed_at_ms: 2_002,
            })
            .unwrap();
        assert_eq!(
            engine.state().wallet.runtime.outbound_transfers.records.len(),
            1
        );
        assert!(
            engine.state().wallet.runtime.outbound_transfers.records[0].paired
        );
    }
}

#[cfg(test)]
mod wallet_sponsored_fee_engine_tests {
    use super::*;

    fn identity(address: &str, revision: u64) -> WalletTransferIdentity {
        WalletTransferIdentity {
            network: 1,
            address: address.into(),
            public_key: vec![5; 32],
            revision,
        }
    }

    #[test]
    fn engine_wires_sponsored_fee_request_response_and_failure_fences() {
        let mut engine = MessagingEngine::new();
        engine
            .execute(Command::ResetWalletSponsoredFeeGeneration {
                network_generation: 7,
            })
            .unwrap();
        let who = identity("EQ-wallet", 4);
        engine
            .execute(Command::BeginWalletSponsoredFeeRequest {
                network_generation: 7,
                identity: who.clone(),
                transfer_min_nano: 100_000_000,
                configured_min_nano: 200_000_000,
                observed_at_ms: 1_000,
                force: false,
            })
            .unwrap();
        let serial = engine
            .state()
            .wallet
            .runtime
            .sponsored_fees
            .active_request
            .as_ref()
            .unwrap()
            .serial;

        assert_eq!(
            engine.execute(Command::ApplyWalletSponsoredFeeInfo {
                serial,
                network_generation: 8,
                identity: who.clone(),
                info: WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 300_000_000,
                    reset_at_ms: 100_000,
                    left: 1,
                    available: true,
                },
                observed_at_ms: 2_000,
            }),
            Err(EngineError::WalletSponsoredFee(
                WalletSponsoredFeeError::StaleNetworkGeneration {
                    current: 7,
                    received: 8,
                }
            ))
        );

        engine
            .execute(Command::ApplyWalletSponsoredFeeInfo {
                serial,
                network_generation: 7,
                identity: who.clone(),
                info: WalletSponsoredFeeInfo {
                    relayer_address: "EQ-relayer".into(),
                    min_amount_nano: 300_000_000,
                    reset_at_ms: 100_000,
                    left: 1,
                    available: true,
                },
                observed_at_ms: 2_000,
            })
            .unwrap();
        assert_eq!(
            engine.state().wallet.runtime.sponsored_fees.terms.effective_min_nano,
            300_000_000
        );
        assert!(engine.state().wallet.runtime.sponsored_fees.terms.usable);

        engine
            .execute(Command::BeginWalletSponsoredFeeRequest {
                network_generation: 7,
                identity: who.clone(),
                transfer_min_nano: 100_000_000,
                configured_min_nano: 200_000_000,
                observed_at_ms: 17_000,
                force: true,
            })
            .unwrap();
        let second = engine
            .state()
            .wallet
            .runtime
            .sponsored_fees
            .active_request
            .as_ref()
            .unwrap()
            .serial;
        engine
            .execute(Command::FailWalletSponsoredFeeRequest {
                serial: second,
                network_generation: 7,
                identity: who,
                observed_at_ms: 17_001,
            })
            .unwrap();
        assert!(!engine.state().wallet.runtime.sponsored_fees.terms.fresh);
        assert!(!engine.state().wallet.runtime.sponsored_fees.terms.usable);

        let serialized = serde_json::to_string(engine.state()).unwrap();
        assert!(!serialized.contains("sponsoredFees"));
    }
}

#[cfg(test)]
mod wallet_live_engine_tests {
    use super::*;

    #[test]
    fn engine_fences_stale_wallet_live_generations_and_recovers_from_failures() {
        let mut engine = MessagingEngine::new();
        engine.execute(Command::BeginWalletLiveGeneration).unwrap();
        let first = engine.state().wallet.runtime.live.generation;
        assert_eq!(first, 1);

        engine
            .execute(Command::ReconcileWalletLivePresence {
                generation: first,
                presence: WalletLivePresence::Ready,
                observed_at_ms: 1_000,
            })
            .unwrap();
        engine
            .execute(Command::MarkWalletLiveStateFailed {
                generation: first,
                observed_at_ms: 2_000,
            })
            .unwrap();
        assert!(!engine.state().wallet.runtime.live.state_unreachable);
        engine
            .execute(Command::MarkWalletLiveStateFailed {
                generation: first,
                observed_at_ms: 3_000,
            })
            .unwrap();
        assert!(engine.state().wallet.runtime.live.state_unreachable);

        engine.execute(Command::BeginWalletLiveGeneration).unwrap();
        let second = engine.state().wallet.runtime.live.generation;
        assert_eq!(second, 2);
        assert_eq!(
            engine.execute(Command::MarkWalletLiveStateFailed {
                generation: first,
                observed_at_ms: 4_000,
            }),
            Err(EngineError::WalletLive(WalletLiveError::StaleGeneration {
                current: second,
                received: first,
            }))
        );
        engine
            .execute(Command::MarkWalletStreamResynced {
                generation: second,
                observed_at_ms: 5_000,
            })
            .unwrap();
        assert!(!engine.state().wallet.runtime.live.stream_resync_due(
            5_000 + crate::wallet::WALLET_LIVE_STREAM_RESYNC_MS - 1
        ));
    }

    #[test]
    fn engine_bounds_hidden_wallet_history_without_persisting_live_authority() {
        let mut engine = MessagingEngine::new();
        engine.execute(Command::BeginWalletLiveGeneration).unwrap();
        let generation = engine.state().wallet.runtime.live.generation;
        for _ in 0..crate::wallet::WALLET_LIVE_MAX_HIDDEN_PAGES {
            engine
                .execute(Command::SpendWalletHistoryPageRequest { generation })
                .unwrap();
            engine
                .execute(Command::RecordWalletHistoryProgress {
                    generation,
                    visible_rows: 0,
                })
                .unwrap();
        }
        assert_eq!(
            engine.execute(Command::SpendWalletHistoryPageRequest { generation }),
            Err(EngineError::WalletLive(
                WalletLiveError::HistoryPageBudgetExhausted
            ))
        );
        assert_eq!(
            engine.state().wallet.runtime.live.hidden_pages_without_visible_rows,
            crate::wallet::WALLET_LIVE_MAX_HIDDEN_PAGES
        );
        engine
            .execute(Command::RearmWalletHistoryWalk { generation })
            .unwrap();
        assert_eq!(
            engine.state().wallet.runtime.live.hidden_pages_without_visible_rows,
            0
        );

        let serialized = serde_json::to_string(engine.state()).unwrap();
        let restored: MessagingState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored.wallet.runtime.live.generation, 0);
    }
}
