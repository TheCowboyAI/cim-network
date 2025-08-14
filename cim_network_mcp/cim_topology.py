"""
CIM (Composable Information Machine) Topology Implementation

This module implements the proper CIM hierarchical architecture:
DEV/CLIENT -> LEAF -> cluster -> super-cluster

Using NATS lattice as the connective message bus with event-sourcing.
"""

from dataclasses import dataclass, field
from typing import Dict, List, Set, Optional, Any, Union
from enum import Enum
import uuid
import json
from datetime import datetime
import hashlib


# CIM Identifiers
@dataclass(frozen=True)
class CimId:
    """Base class for all CIM identifiers"""
    value: str = field(default_factory=lambda: str(uuid.uuid4()))

@dataclass(frozen=True)
class ClientId(CimId):
    """Client identifier"""
    pass

@dataclass(frozen=True)
class LeafId(CimId):
    """Leaf node identifier"""
    pass

@dataclass(frozen=True)
class ClusterId(CimId):
    """Cluster identifier"""
    pass

@dataclass(frozen=True)
class SuperClusterId(CimId):
    """Super-cluster identifier"""
    pass

@dataclass(frozen=True)
class TopologyId(CimId):
    """Topology identifier"""
    pass

@dataclass(frozen=True)
class CorrelationId(CimId):
    """Correlation identifier for event chains"""
    pass

@dataclass(frozen=True)
class CausationId(CimId):
    """Causation identifier for event causation"""
    pass

@dataclass(frozen=True)
class Cid(CimId):
    """Content identifier for events (content-addressed)"""
    pass


# CIM Tiers
class CimTier(Enum):
    """CIM hierarchical tiers"""
    CLIENT = "client"
    LEAF = "leaf"
    CLUSTER = "cluster"
    SUPER_CLUSTER = "super_cluster"


class ClientType(Enum):
    """Types of CIM clients"""
    DEVELOPER = "developer"
    APPLICATION = "application"
    SERVICE = "service"
    BROWSER = "browser"
    CLI = "cli"


# NATS Configuration
@dataclass
class NatsSecurityConfig:
    """NATS security configuration"""
    tls_enabled: bool = True
    auth_required: bool = True
    jwt_enabled: bool = True
    nkeys_enabled: bool = True

@dataclass
class NatsGatewayConfig:
    """NATS gateway configuration for super-clusters"""
    name: str
    host: str
    port: int = 7222
    tls: bool = True
    authorization: Dict[str, Any] = field(default_factory=dict)

@dataclass
class NatsClusterConfig:
    """NATS cluster configuration"""
    name: str
    routes: List[str] = field(default_factory=list)
    cluster_port: int = 6222
    jetstream: bool = True

@dataclass
class NatsLeafConfig:
    """NATS leaf configuration"""
    name: str
    remotes: List[str] = field(default_factory=list)
    leaf_port: int = 7422

@dataclass
class JetStreamConfig:
    """JetStream configuration for event persistence"""
    enabled: bool = True
    max_memory: str = "1GB"
    max_file: str = "10GB"
    store_dir: str = "/var/lib/nats/jetstream"

@dataclass
class NatsLatticeConfig:
    """Complete NATS lattice configuration"""
    gateways: List[NatsGatewayConfig] = field(default_factory=list)
    clusters: List[NatsClusterConfig] = field(default_factory=list)
    leaves: List[NatsLeafConfig] = field(default_factory=list)
    jetstream: JetStreamConfig = field(default_factory=JetStreamConfig)
    security: NatsSecurityConfig = field(default_factory=NatsSecurityConfig)

    @classmethod
    def development(cls) -> 'NatsLatticeConfig':
        """Development NATS configuration (single node)"""
        return cls(
            clusters=[NatsClusterConfig(name="dev-cluster")],
            leaves=[NatsLeafConfig(name="dev-leaf", remotes=["nats://localhost:4222"])],
            jetstream=JetStreamConfig(
                max_memory="256MB",
                max_file="1GB",
                store_dir="/tmp/nats-dev"
            )
        )

    @classmethod
    def production(cls) -> 'NatsLatticeConfig':
        """Production NATS configuration (distributed)"""
        return cls(
            gateways=[
                NatsGatewayConfig(name="super-gateway", host="0.0.0.0"),
            ],
            clusters=[
                NatsClusterConfig(
                    name="primary-cluster",
                    routes=["nats://cluster1:6222", "nats://cluster2:6222"]
                ),
            ],
            jetstream=JetStreamConfig(
                max_memory="8GB",
                max_file="100GB"
            )
        )


# CIM Event Sourcing
@dataclass
class VectorClock:
    """Vector clock for distributed event ordering"""
    clocks: Dict[str, int] = field(default_factory=dict)

    def tick(self, node_id: str) -> None:
        """Increment clock for node"""
        self.clocks[node_id] = self.clocks.get(node_id, 0) + 1

    def update(self, other: 'VectorClock') -> None:
        """Update with another vector clock"""
        for node_id, clock in other.clocks.items():
            self.clocks[node_id] = max(self.clocks.get(node_id, 0), clock)

@dataclass
class CimEvent:
    """CIM event with content-addressing and causation chains"""
    event_cid: Cid
    previous_cid: Optional[Cid]
    correlation_id: CorrelationId
    causation_id: CausationId
    source_tier: CimTier
    node_id: str
    payload: Dict[str, Any]
    timestamp: datetime
    vector_clock: VectorClock

    @classmethod
    def create(cls, payload: Dict[str, Any], source_tier: CimTier, 
               node_id: str, previous_cid: Optional[Cid] = None,
               correlation_id: Optional[CorrelationId] = None,
               causation_id: Optional[CausationId] = None) -> 'CimEvent':
        """Create new CIM event with proper addressing"""
        
        # Generate content-addressed ID
        content = {
            "payload": payload,
            "source_tier": source_tier.value,
            "node_id": node_id,
            "timestamp": datetime.utcnow().isoformat(),
            "previous_cid": previous_cid.value if previous_cid else None
        }
        content_bytes = json.dumps(content, sort_keys=True).encode()
        event_cid = Cid(hashlib.sha256(content_bytes).hexdigest())

        return cls(
            event_cid=event_cid,
            previous_cid=previous_cid,
            correlation_id=correlation_id or CorrelationId(),
            causation_id=causation_id or CausationId(),
            source_tier=source_tier,
            node_id=node_id,
            payload=payload,
            timestamp=datetime.utcnow(),
            vector_clock=VectorClock()
        )


# CIM Node Configurations
@dataclass
class ClientConfig:
    """Client configuration in CIM topology"""
    client_id: ClientId
    name: str
    client_type: ClientType
    assigned_leaf: LeafId
    auth_config: Dict[str, Any] = field(default_factory=dict)
    rate_limits: Dict[str, Any] = field(default_factory=dict)

@dataclass
class LeafConfig:
    """Leaf node configuration"""
    leaf_id: LeafId
    name: str
    cluster: ClusterId
    nats_leaf: NatsLeafConfig
    event_store_config: Dict[str, Any] = field(default_factory=dict)
    assigned_clients: Set[ClientId] = field(default_factory=set)
    resource_limits: Dict[str, Any] = field(default_factory=dict)

@dataclass
class ClusterConfig:
    """Cluster configuration"""
    cluster_id: ClusterId
    name: str
    domain: str  # Domain boundary
    super_cluster: SuperClusterId
    nats_cluster: NatsClusterConfig
    managed_leaves: Set[LeafId] = field(default_factory=set)
    saga_definitions: Dict[str, Any] = field(default_factory=dict)
    projection_configs: List[Dict[str, Any]] = field(default_factory=list)

@dataclass
class SuperClusterConfig:
    """Super-cluster configuration"""
    super_id: SuperClusterId
    name: str
    nats_gateway: NatsGatewayConfig
    managed_clusters: Set[ClusterId] = field(default_factory=set)
    global_projections: List[Dict[str, Any]] = field(default_factory=list)
    orchestration_rules: List[Dict[str, Any]] = field(default_factory=list)


# CIM Topology
@dataclass
class CimTopology:
    """Complete CIM hierarchical topology"""
    topology_id: TopologyId
    super_clusters: Dict[SuperClusterId, SuperClusterConfig] = field(default_factory=dict)
    clusters: Dict[ClusterId, ClusterConfig] = field(default_factory=dict)
    leaves: Dict[LeafId, LeafConfig] = field(default_factory=dict)
    clients: Dict[ClientId, ClientConfig] = field(default_factory=dict)
    nats_config: NatsLatticeConfig = field(default_factory=NatsLatticeConfig)
    genesis_cid: Cid = field(default_factory=lambda: Cid(value="genesis"))
    version: int = 0

    def get_tier_summary(self) -> Dict[str, Any]:
        """Get summary of all tiers in topology"""
        return {
            "topology_id": self.topology_id.value,
            "version": self.version,
            "tiers": {
                "super_clusters": len(self.super_clusters),
                "clusters": len(self.clusters),
                "leaves": len(self.leaves),
                "clients": len(self.clients)
            },
            "hierarchy": self._build_hierarchy_view()
        }

    def _build_hierarchy_view(self) -> Dict[str, Any]:
        """Build hierarchical view of topology"""
        hierarchy = {}
        
        for super_id, super_config in self.super_clusters.items():
            super_clusters = hierarchy.setdefault("super_clusters", {})
            super_clusters[super_id.value] = {
                "name": super_config.name,
                "clusters": {}
            }
            
            for cluster_id in super_config.managed_clusters:
                if cluster_id in self.clusters:
                    cluster_config = self.clusters[cluster_id]
                    super_clusters[super_id.value]["clusters"][cluster_id.value] = {
                        "name": cluster_config.name,
                        "domain": cluster_config.domain,
                        "leaves": {}
                    }
                    
                    for leaf_id in cluster_config.managed_leaves:
                        if leaf_id in self.leaves:
                            leaf_config = self.leaves[leaf_id]
                            super_clusters[super_id.value]["clusters"][cluster_id.value]["leaves"][leaf_id.value] = {
                                "name": leaf_config.name,
                                "clients": [client_id.value for client_id in leaf_config.assigned_clients]
                            }
        
        return hierarchy


# CIM Commands and Events
class CimCommand:
    """Base class for CIM commands"""
    pass

@dataclass
class InitializeTopology(CimCommand):
    """Initialize new CIM topology"""
    name: str
    initial_super_cluster: SuperClusterConfig

@dataclass
class AddCluster(CimCommand):
    """Add new cluster to topology"""
    cluster_config: ClusterConfig
    super_cluster_id: SuperClusterId

@dataclass
class AddLeaf(CimCommand):
    """Add new leaf node"""
    leaf_config: LeafConfig
    cluster_id: ClusterId

@dataclass
class RegisterClient(CimCommand):
    """Register client"""
    client_config: ClientConfig
    preferred_leaf: Optional[LeafId] = None

@dataclass
class ScaleCluster(CimCommand):
    """Scale cluster (add/remove leaves)"""
    cluster_id: ClusterId
    target_leaf_count: int

@dataclass
class MigrateClient(CimCommand):
    """Migrate client to different leaf"""
    client_id: ClientId
    target_leaf: LeafId
    migration_strategy: str = "graceful"


class CimTopologyEvent:
    """Base class for CIM topology events"""
    pass

@dataclass
class TopologyInitialized(CimTopologyEvent):
    """Topology initialized event"""
    topology_id: TopologyId
    super_cluster: SuperClusterConfig
    genesis_cid: Cid

@dataclass
class ClusterAdded(CimTopologyEvent):
    """Cluster added event"""
    cluster_id: ClusterId
    super_cluster_id: SuperClusterId
    nats_config: NatsClusterConfig

@dataclass
class LeafAdded(CimTopologyEvent):
    """Leaf added event"""
    leaf_id: LeafId
    cluster_id: ClusterId
    nats_config: NatsLeafConfig

@dataclass
class ClientRegistered(CimTopologyEvent):
    """Client registered event"""
    client_id: ClientId
    assigned_leaf: LeafId
    auth_token: str


# CIM Topology Builder
class CimTopologyBuilder:
    """Builder for CIM topologies"""
    
    def __init__(self):
        self.topology = CimTopology(topology_id=TopologyId())

    @classmethod
    def development(cls, name: str) -> 'CimTopologyBuilder':
        """Create development topology (single leaf, local NATS)"""
        builder = cls()
        
        # Create super-cluster
        super_id = SuperClusterId(value="dev-super")
        super_config = SuperClusterConfig(
            super_id=super_id,
            name=f"{name} Development Super-cluster",
            nats_gateway=NatsGatewayConfig(name="dev-gateway", host="localhost")
        )
        
        # Create cluster
        cluster_id = ClusterId(value="dev-cluster")
        cluster_config = ClusterConfig(
            cluster_id=cluster_id,
            name=f"{name} Development Cluster",
            domain="development",
            super_cluster=super_id,
            nats_cluster=NatsClusterConfig(name="dev-cluster")
        )
        
        # Create leaf
        leaf_id = LeafId(value="dev-leaf")
        leaf_config = LeafConfig(
            leaf_id=leaf_id,
            name=f"{name} Development Leaf",
            cluster=cluster_id,
            nats_leaf=NatsLeafConfig(name="dev-leaf")
        )
        
        # Wire up relationships
        super_config.managed_clusters.add(cluster_id)
        cluster_config.managed_leaves.add(leaf_id)
        
        # Add to topology
        builder.topology.super_clusters[super_id] = super_config
        builder.topology.clusters[cluster_id] = cluster_config
        builder.topology.leaves[leaf_id] = leaf_config
        builder.topology.nats_config = NatsLatticeConfig.development()
        
        return builder

    @classmethod
    def production(cls, name: str) -> 'CimTopologyBuilder':
        """Create production topology (multi-cluster, distributed)"""
        builder = cls()
        
        # Create super-cluster
        super_id = SuperClusterId(value="prod-super")
        super_config = SuperClusterConfig(
            super_id=super_id,
            name=f"{name} Production Super-cluster",
            nats_gateway=NatsGatewayConfig(name="prod-gateway", host="0.0.0.0")
        )
        
        # Create regional clusters
        regions = ["us-east", "us-west", "eu-west"]
        for region in regions:
            cluster_id = ClusterId(value=f"{region}-cluster")
            cluster_config = ClusterConfig(
                cluster_id=cluster_id,
                name=f"{name} {region.title()} Cluster",
                domain=region,
                super_cluster=super_id,
                nats_cluster=NatsClusterConfig(name=f"{region}-cluster")
            )
            
            # Add leaves to each cluster
            for i in range(3):  # 3 leaves per cluster
                leaf_id = LeafId(value=f"{region}-leaf-{i+1}")
                leaf_config = LeafConfig(
                    leaf_id=leaf_id,
                    name=f"{name} {region.title()} Leaf {i+1}",
                    cluster=cluster_id,
                    nats_leaf=NatsLeafConfig(name=f"{region}-leaf-{i+1}")
                )
                
                cluster_config.managed_leaves.add(leaf_id)
                builder.topology.leaves[leaf_id] = leaf_config
            
            super_config.managed_clusters.add(cluster_id)
            builder.topology.clusters[cluster_id] = cluster_config
        
        builder.topology.super_clusters[super_id] = super_config
        builder.topology.nats_config = NatsLatticeConfig.production()
        
        return builder

    def with_client(self, name: str, client_type: ClientType, 
                   preferred_leaf: Optional[LeafId] = None) -> 'CimTopologyBuilder':
        """Add client to topology"""
        client_id = ClientId()
        
        # Assign to leaf (prefer specified, otherwise round-robin)
        if preferred_leaf and preferred_leaf in self.topology.leaves:
            assigned_leaf = preferred_leaf
        else:
            # Simple round-robin assignment
            leaves = list(self.topology.leaves.keys())
            if leaves:
                assigned_leaf = leaves[len(self.topology.clients) % len(leaves)]
            else:
                raise ValueError("No leaves available for client assignment")
        
        client_config = ClientConfig(
            client_id=client_id,
            name=name,
            client_type=client_type,
            assigned_leaf=assigned_leaf
        )
        
        self.topology.clients[client_id] = client_config
        self.topology.leaves[assigned_leaf].assigned_clients.add(client_id)
        
        return self

    def build(self) -> CimTopology:
        """Build final topology"""
        self.topology.version = 1
        return self.topology


def create_development_cim(name: str) -> CimTopology:
    """Create development CIM topology"""
    return (CimTopologyBuilder
            .development(name)
            .with_client("Developer CLI", ClientType.CLI)
            .with_client("Local Browser", ClientType.BROWSER)
            .build())


def create_production_cim(name: str) -> CimTopology:
    """Create production CIM topology"""
    builder = CimTopologyBuilder.production(name)
    
    # Add various client types
    client_types = [
        ("Web Application", ClientType.APPLICATION),
        ("Mobile Service", ClientType.SERVICE),
        ("Admin CLI", ClientType.CLI),
        ("Developer Workspace", ClientType.DEVELOPER),
    ]
    
    for client_name, client_type in client_types:
        builder.with_client(client_name, client_type)
    
    return builder.build()