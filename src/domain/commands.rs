//! Domain commands

use super::value_objects::*;
use serde::{Deserialize, Serialize};

/// Network commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkCommand {
    /// Add a router
    AddRouter {
        router_id: RouterId,
    },
}