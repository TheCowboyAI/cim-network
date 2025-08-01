//! Domain events

use super::value_objects::*;
use serde::{Deserialize, Serialize};

/// Network events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// Router was added
    RouterAdded {
        router_id: RouterId,
    },
}