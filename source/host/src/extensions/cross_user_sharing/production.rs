use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

use base64::Engine;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use serde_json::Value;

use crate::connectors::channel_attachment::ResolvedChannelAttachment;
use crate::extensions::attachments::attachments_service::AttachmentsService;
use crate::extensions::auth::credential_renewer::get_configured_backend_url;
use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::experiments::HostExperimentsExtension;
use crate::extensions::notify_bus::extension::HostNotifyBusExtension;
use crate::extensions::transcript::send_group_fanout::GroupMemberTurnExecutor;
use crate::extensions::transcript::shared_rooms::SharedRooms;
use crate::groups::group_store::RemoteGroupMember;
use crate::groups::remote_room_store::RemoteRoomMember;
use crate::host_paths::get_sand_root_dir;

use super::extension::CrossUserSharingExtension;
use super::xuser_entry_publisher::{
    ResolvedXuserAttachment, SandXuserEntryPublisher, XuserEntryPublisherHost,
};
use super::xuser_relay::{
    ReqwestXuserRelayTransport, SandXuserRelayClient, SandXuserRelayDriver,
    SandXuserRelayRuntime, XuserRelayEventHandler, XuserRelayTransport,
};
use super::xuser_remote_turns::{
    AgentDisplayProfile, RemoteTurnsHost, SandXuserRemoteTurns,
};
use super::xuser_sharing_service::{SandXuserSharingService, XuserSharingManager};
use super::xuser_turn_dedupe_store::{
    SandXuserTurnDedupeStore, XUSER_TURN_DEDUPE_TTL_MS, XuserTurnDedupe,
};

pub type RemoteRequestedTurnRunner =
    Arc<dyn Fn(&str, &str, &str) -> Result<Vec<String>, String> + Send + Sync>;
pub type SharedRoomTurnRunner = Arc<
    dyn Fn(&str, Option<GroupMemberTurnExecutor>) -> Result<(), String> + Send + Sync,
>;

/// Shipping CrossUserSharing owner.
///
/// This deliberately composes the frozen Grok dependency direction:
/// Auth + Attachments + Experiments + Transcript + NotifyBus -> CrossUserSharing.
/// Remote requested turns are delegated back to the canonical Host Runner through
/// `run_remote_requested_turn`; this module never constructs a second provider
/// or Runner runtime.
pub struct ProductionCrossUserRuntime {
    extension: Arc<CrossUserSharingExtension>,
    service: Arc<SandXuserSharingService>,
    remote_turns: Arc<SandXuserRemoteTurns>,
    auth: Arc<HostAuthExtension>,
    auth_renewal_subscription: u64,
    notify_bus: HostNotifyBusExtension,
}

struct ProductionXuserHost {
    relay: Arc<SandXuserRelayClient>,
    service: Weak<SandXuserSharingService>,
    shared_rooms: Arc<SharedRooms>,
    attachments: Arc<AttachmentsService>,
    run_remote_requested_turn: RemoteRequestedTurnRunner,
    run_shared_room_turn: SharedRoomTurnRunner,
    remote_turns: Arc<Mutex<Weak<SandXuserRemoteTurns>>>,
}

impl ProductionXuserHost {
    fn service(&self) -> Option<Arc<SandXuserSharingService>> {
        self.service.upgrade()
    }
}

impl RemoteTurnsHost for ProductionXuserHost {
    fn relay_send(&self, payload: &Value) -> Result<Value, String> {
        self.relay.send(payload)
    }

    fn relay_request_drain(&self) {
        if let Some(service) = self.service() {
            service.request_relay_drain();
        }
    }

    fn relay_remove_deleted_agent(&self, agent_id: &str) -> Result<Value, String> {
        self.relay.remove_deleted_agent(agent_id)
    }

    fn now_ms(&self) -> u64 {
        now_ms()
    }

    fn is_enabled(&self) -> bool {
        self.service()
            .is_some_and(|service| service.get_state().is_enabled)
    }

    fn self_auth_id(&self) -> Option<String> {
        self.service()
            .and_then(|service| service.get_state().self_auth_id)
    }

    fn rooms(&self) -> Vec<super::xuser_state_reconcile::XuserRoom> {
        self.service()
            .map(|service| service.get_state().rooms)
            .unwrap_or_default()
    }

    fn get_agent_display_profile(
        &self,
        agent_id: &str,
    ) -> Result<Option<AgentDisplayProfile>, String> {
        self.shared_rooms
            .sessions()
            .summarize_agent_by_id(agent_id, None)
            .map(|summary| {
                summary.map(|summary| AgentDisplayProfile {
                    name: summary.name,
                    description: summary.description,
                })
            })
    }

    fn append_shared_room_activity_notice(
        &self,
        room_id: &str,
        text: &str,
    ) -> Result<(), String> {
        self.shared_rooms
            .append_shared_room_activity_notice(room_id, text)
            .map(|_| ())
    }

    fn run_remote_requested_member_turn(
        &self,
        agent_id: &str,
        system_prompt: &str,
        prompt: &str,
    ) -> Result<Vec<String>, String> {
        (self.run_remote_requested_turn)(agent_id, system_prompt, prompt)
    }

    fn reconcile_share_state(&self) -> Result<(), String> {
        let Some(service) = self.service() else {
            return Ok(());
        };
        service.reconcile_share_state().map(|_| ())
    }
}

impl XuserSharingManager for ProductionXuserHost {
    fn install_room(
        &self,
        room: &super::xuser_state_reconcile::XuserRoom,
        self_auth_id: Option<&str>,
    ) -> Result<(), String> {
        let Some(self_auth_id) = self_auth_id else {
            return Ok(());
        };
        if room.host_auth_id == self_auth_id {
            let local_member_ids = room
                .members
                .iter()
                .filter(|member| member.auth_id == self_auth_id)
                .filter_map(|member| member.agent_id.clone())
                .collect::<Vec<_>>();
            let remote_members = room
                .members
                .iter()
                .filter(|member| member.auth_id != self_auth_id)
                .filter_map(|member| {
                    Some(RemoteGroupMember {
                        owner_auth_id: member.auth_id.clone(),
                        agent_id: member.agent_id.clone()?,
                        name: member
                            .name
                            .clone()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| "Agent".into()),
                        avatar_data_url: None,
                    })
                })
                .collect::<Vec<_>>();
            self.shared_rooms
                .ensure_hosted_shared_room(
                    &room.room_id,
                    &room.name,
                    &local_member_ids,
                    &remote_members,
                    true,
                )
                .map(|_| ())
        } else {
            let host_name = room
                .members
                .iter()
                .find(|member| {
                    member.auth_id == room.host_auth_id && member.agent_id.is_none()
                })
                .and_then(|member| member.name.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "The host".into());
            let members = room
                .members
                .iter()
                .filter(|member| member.auth_id != self_auth_id)
                .map(|member| RemoteRoomMember {
                    kind: if member.agent_id.is_some() {
                        "agent".into()
                    } else {
                        "human".into()
                    },
                    auth_id: member.auth_id.clone(),
                    agent_id: member.agent_id.clone().unwrap_or_default(),
                    display_name: member
                        .name
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| {
                            if member.agent_id.is_some() {
                                "Agent".into()
                            } else {
                                "Someone".into()
                            }
                        }),
                    avatar_url: None,
                })
                .collect::<Vec<_>>();
            self.shared_rooms
                .ensure_mirror_room(
                    &room.room_id,
                    &room.name,
                    &room.host_auth_id,
                    &host_name,
                    None,
                    &members,
                    self_auth_id,
                )
                .map(|_| ())
        }
    }

    fn mark_mirror_room_revoked(&self, room_id: &str) -> Result<(), String> {
        self.shared_rooms.mark_mirror_room_revoked(room_id)
    }

    fn post_shared_room_guest_message(&self, event: &Value) -> Result<(), String> {
        let Some(room_agent_id) = self.shared_rooms.post_shared_room_guest_message(event)? else {
            return Ok(());
        };
        let remote_executor = self
            .remote_turns
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .upgrade()
            .map(|turns| {
                Arc::new(move |request| {
                    let Some(shared_room_id) = request.shared_room_id.as_deref() else {
                        return Ok(Vec::new());
                    };
                    Ok(turns.run_remote_member_turn(
                        &request.member,
                        shared_room_id,
                        &request.group,
                        &request.peers,
                        &request.new_messages,
                    ))
                }) as GroupMemberTurnExecutor
            });
        (self.run_shared_room_turn)(&room_agent_id, remote_executor)
    }

    fn append_mirror_room_entry(
        &self,
        event: &Value,
        self_auth_id: &str,
    ) -> Result<bool, String> {
        self.shared_rooms
            .append_mirror_room_entry(event, self_auth_id)
    }
}

impl XuserEntryPublisherHost for ProductionXuserHost {
    fn relay_send(&self, payload: &Value) -> Result<Value, String> {
        self.relay.send(payload)
    }

    fn restamp_room_entry(
        &self,
        room_id: &str,
        entry_id: &str,
        timestamp_ms: f64,
    ) -> Result<(), String> {
        self.shared_rooms
            .restamp_room_entry(room_id, entry_id, timestamp_ms)
            .map(|_| ())
    }

    fn now_ms(&self) -> u64 {
        now_ms()
    }

    fn is_enabled(&self) -> bool {
        self.service()
            .is_some_and(|service| service.get_state().is_enabled)
    }

    fn self_auth_id(&self) -> Option<String> {
        self.service()
            .and_then(|service| service.get_state().self_auth_id)
    }

    fn resolve_attachment(&self, url: &str) -> Result<Option<ResolvedXuserAttachment>, String> {
        Ok(match self.attachments.resolve_channel_attachment(Some(url)) {
            Some(ResolvedChannelAttachment::Upload {
                is_image: true,
                bytes,
                mime,
                ..
            }) if mime.starts_with("image/") => Some(ResolvedXuserAttachment {
                data: bytes,
                mime_type: mime,
            }),
            _ => None,
        })
    }
}

impl ProductionCrossUserRuntime {
    pub fn new(
        auth: Arc<HostAuthExtension>,
        notify_bus: HostNotifyBusExtension,
        attachments: Arc<AttachmentsService>,
        shared_rooms: Arc<SharedRooms>,
        run_remote_requested_turn: RemoteRequestedTurnRunner,
        run_shared_room_turn: SharedRoomTurnRunner,
    ) -> Result<Arc<Self>, String> {
        let backend_url = get_configured_backend_url().map_err(|error| error.to_string())?;
        let token_auth = Arc::clone(&auth);
        let transport: Arc<dyn XuserRelayTransport> =
            Arc::new(ReqwestXuserRelayTransport::new()?);
        let relay = Arc::new(SandXuserRelayClient::new(
            backend_url.clone(),
            Arc::new(move || {
                token_auth
                    .get_access_token()
                    .map_err(|error| error.to_string())
            }),
            transport,
        ));
        let service = Arc::new(SandXuserSharingService::new(Arc::clone(&relay)));
        service.set_self_auth_id(
            auth.peek_access_token()
                .as_deref()
                .and_then(jwt_subject_from_access_token),
        );

        let remote_turns_slot = Arc::new(Mutex::new(Weak::<SandXuserRemoteTurns>::new()));
        let host = Arc::new(ProductionXuserHost {
            relay: Arc::clone(&relay),
            service: Arc::downgrade(&service),
            shared_rooms,
            attachments,
            run_remote_requested_turn,
            run_shared_room_turn,
            remote_turns: Arc::clone(&remote_turns_slot),
        });
        let remote_host: Arc<dyn RemoteTurnsHost> = host.clone();
        let manager_host: Arc<dyn XuserSharingManager> = host.clone();
        let publisher_host: Arc<dyn XuserEntryPublisherHost> = host;
        service.bind_manager(manager_host);
        let dedupe: Arc<dyn XuserTurnDedupe> = Arc::new(SandXuserTurnDedupeStore::new(
            &get_sand_root_dir(),
            XUSER_TURN_DEDUPE_TTL_MS,
            Box::new(now_ms),
        ));
        let remote_turns = Arc::new(SandXuserRemoteTurns::new(remote_host, dedupe));
        service.bind_entry_publisher(Arc::new(SandXuserEntryPublisher::new(
            publisher_host,
        )));

        // The relay owns polling/ack/backoff. Event dispatch stays inside the
        // CrossUserSharing boundary and delegates room state to Transcript-owned
        // SharedRooms while turn execution remains on the canonical Host Runner.
        *remote_turns_slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Arc::downgrade(&remote_turns);
        let relay_turns = Arc::clone(&remote_turns_slot);
        let relay_service = Arc::downgrade(&service);
        let on_event: XuserRelayEventHandler = Arc::new(move |event| {
            let Some(turns) = relay_turns
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .upgrade()
            else {
                return Ok(false);
            };
            match event.get("kind").and_then(Value::as_str) {
                Some("turn-request") => Ok(turns.handle_turn_request(event)),
                Some("turn-result") => Ok(turns.handle_turn_result(event)),
                _ => {
                    let Some(service) = relay_service.upgrade() else {
                        return Ok(false);
                    };
                    service.handle_event(event)
                }
            }
        });
        let notify_connected = notify_bus.clone();
        let notify_safety = notify_bus.clone();
        let relay_runtime = Arc::new(SandXuserRelayRuntime::new(
            Arc::clone(&relay),
            on_event,
            Arc::new(now_ms),
            Arc::new(move || notify_connected.is_connected()),
            Arc::new(move || notify_safety.is_safety_poll_enabled()),
        ));
        service.bind_relay_driver(Arc::new(SandXuserRelayDriver::new(relay_runtime)));

        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        let extension = Arc::new(CrossUserSharingExtension::new(
            Arc::clone(&service),
            &backend_url,
            &env,
        ));

        let renewal_service = Arc::downgrade(&service);
        let renewal_auth = Arc::downgrade(&auth);
        let auth_renewal_subscription = auth.service().subscribe_to_renewal(Arc::new(move |_| {
            let Some(service) = renewal_service.upgrade() else {
                return;
            };
            let subject = renewal_auth
                .upgrade()
                .and_then(|auth| auth.peek_access_token())
                .as_deref()
                .and_then(jwt_subject_from_access_token);
            service.set_self_auth_id(subject);
        }));

        Ok(Arc::new(Self {
            extension,
            service,
            remote_turns,
            auth,
            auth_renewal_subscription,
            notify_bus,
        }))
    }

    pub fn start_background_work(
        self: &Arc<Self>,
        experiments: Arc<HostExperimentsExtension>,
    ) -> Result<(), String> {
        self.extension
            .start_background_work(experiments, self.notify_bus.clone())
    }

    pub fn is_enabled(&self) -> bool {
        self.extension.is_enabled()
    }

    pub fn remote_executor(&self) -> Option<GroupMemberTurnExecutor> {
        if !self.is_enabled() {
            return None;
        }
        let turns = Arc::clone(&self.remote_turns);
        Some(Arc::new(move |request| {
            let Some(shared_room_id) = request.shared_room_id.as_deref() else {
                return Ok(Vec::new());
            };
            Ok(turns.run_remote_member_turn(
                &request.member,
                shared_room_id,
                &request.group,
                &request.peers,
                &request.new_messages,
            ))
        }))
    }

    pub fn service(&self) -> Arc<SandXuserSharingService> {
        Arc::clone(&self.service)
    }

    pub fn stop(&self) {
        self.extension.stop();
        self.remote_turns.stop();
        let _ = self
            .auth
            .service()
            .unsubscribe_from_renewal(self.auth_renewal_subscription);
    }
}

pub fn jwt_subject_from_access_token(token: &str) -> Option<String> {
    let encoded = token.split('.').nth(1)?;
    let payload = URL_SAFE_NO_PAD
        .decode(encoded)
        .or_else(|_| URL_SAFE.decode(encoded))
        .ok()?;
    serde_json::from_slice::<Value>(&payload)
        .ok()?
        .get("sub")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::jwt_subject_from_access_token;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    #[test]
    fn jwt_subject_is_read_without_trusting_other_claims() {
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"auth-user","role":"ignored"}"#);
        let token = format!("header.{payload}.signature");
        assert_eq!(
            jwt_subject_from_access_token(&token).as_deref(),
            Some("auth-user")
        );
        assert!(jwt_subject_from_access_token("not-a-jwt").is_none());
    }
}
