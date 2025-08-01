//! Switch state machine with phantom types

use crate::domain::SwitchId;
use chrono::{DateTime, Utc};
use std::marker::PhantomData;

// State marker types
pub struct Planned;
pub struct Configuring;
pub struct Active;
pub struct Failed;

/// Switch state machine
pub struct SwitchStateMachine<S> {
    id: SwitchId,
    name: String,
    port_count: u32,
    created_at: DateTime<Utc>,
    _state: PhantomData<S>,
}

impl SwitchStateMachine<Planned> {
    /// Create a new switch in planned state
    pub fn new(id: SwitchId, name: String, port_count: u32) -> Self {
        Self {
            id,
            name,
            port_count,
            created_at: Utc::now(),
            _state: PhantomData,
        }
    }
}

// Common methods for all states
impl<S> SwitchStateMachine<S> {
    /// Get the switch name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the port count
    pub fn port_count(&self) -> u32 {
        self.port_count
    }
    
    /// Get the switch ID
    pub fn id(&self) -> SwitchId {
        self.id
    }
}