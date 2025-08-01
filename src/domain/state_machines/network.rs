//! Network state machine with phantom types

use crate::domain::{NetworkId, IpNetwork};
use chrono::{DateTime, Utc};
use std::marker::PhantomData;

// State marker types
pub struct Planning;
pub struct Provisioning;
pub struct Active;
pub struct Failed;

/// Network state machine
pub struct NetworkStateMachine<S> {
    id: NetworkId,
    name: String,
    cidr: IpNetwork,
    created_at: DateTime<Utc>,
    _state: PhantomData<S>,
}

impl NetworkStateMachine<Planning> {
    /// Create a new network in planning state
    pub fn new(id: NetworkId, name: String, cidr: IpNetwork) -> Self {
        Self {
            id,
            name,
            cidr,
            created_at: Utc::now(),
            _state: PhantomData,
        }
    }
}

// Common methods for all states
impl<S> NetworkStateMachine<S> {
    /// Get the network name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the CIDR
    pub fn cidr(&self) -> &IpNetwork {
        &self.cidr
    }
    
    /// Get the network ID
    pub fn id(&self) -> NetworkId {
        self.id
    }
}