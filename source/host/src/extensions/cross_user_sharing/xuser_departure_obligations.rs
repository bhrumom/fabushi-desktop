use super::xuser_pending_departure_store::{SandXuserPendingDepartureStore, XuserDeparture};
use super::xuser_room_tombstone_store::{SandXuserRoomTombstoneStore, XuserRoomTombstone};

pub const ROOM_TOMBSTONE_MIN_RETENTION_MS: u64 = 48 * 60 * 60_000;

pub trait XuserDepartureRelay {
    fn leave_room(&self, room_id: &str) -> Result<(), String>;
    fn remove_deleted_agent(&self, agent_id: &str) -> Result<(), String>;
}
pub struct SandXuserDepartureObligations<'a> {
    pub pending: &'a SandXuserPendingDepartureStore,
    pub tombstones: &'a SandXuserRoomTombstoneStore,
}
impl<'a> SandXuserDepartureObligations<'a> {
    pub fn record_leave(
        &self,
        owner_auth_id: Option<String>,
        room_id: String,
        agent_id: String,
        now_ms: u64,
    ) {
        self.pending.record(XuserDeparture::LeaveRoom {
            owner_auth_id: owner_auth_id.clone(),
            room_id: room_id.clone(),
            agent_id,
        });
        self.tombstones.record(XuserRoomTombstone {
            room_id,
            owner_auth_id,
            torn_down_at_ms: now_ms,
        });
    }
    pub fn record_agent_removal(&self, owner_auth_id: Option<String>, agent_id: String) {
        self.pending.record(XuserDeparture::RemoveAgent {
            owner_auth_id,
            agent_id,
        })
    }
    pub fn drain(&self, relay: &dyn XuserDepartureRelay) -> usize {
        let mut completed = 0;
        for departure in self.pending.list() {
            let result = match &departure {
                XuserDeparture::LeaveRoom { room_id, .. } => relay.leave_room(room_id),
                XuserDeparture::RemoveAgent { agent_id, .. } => {
                    relay.remove_deleted_agent(agent_id)
                }
            };
            if result.is_ok() {
                self.pending.clear(&departure);
                completed += 1;
            }
        }
        completed
    }
    pub fn prune_tombstones(&self, now_ms: u64) {
        for t in self.tombstones.list() {
            if now_ms.saturating_sub(t.torn_down_at_ms) >= ROOM_TOMBSTONE_MIN_RETENTION_MS {
                self.tombstones
                    .clear(&t.room_id, t.owner_auth_id.as_deref());
            }
        }
    }
}
