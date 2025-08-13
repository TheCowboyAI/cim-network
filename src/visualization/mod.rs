//! Network Topology Visualization
//!
//! This module provides visualization capabilities for network topologies,
//! generating diagrams and interactive representations of the SDN infrastructure.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::sdn::{SDNNode, SDNConnection};

/// Visualization configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Output format (svg, png, html, ascii)
    pub format: VisualizationFormat,
    /// Layout algorithm (hierarchical, force-directed, circular)
    pub layout: LayoutAlgorithm,
    /// Color scheme (default, dark, blue, enterprise)
    pub color_scheme: ColorScheme,
    /// Show node labels
    pub show_labels: bool,
    /// Show connection properties
    pub show_connection_details: bool,
    /// Group nodes by tier
    pub group_by_tier: bool,
}

/// Available visualization formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationFormat {
    /// SVG vector graphics
    Svg,
    /// PNG raster image
    Png,
    /// Interactive HTML with JavaScript
    Html,
    /// ASCII art for terminal display
    Ascii,
    /// Graphviz DOT format
    Dot,
    /// Mermaid diagram format
    Mermaid,
}

/// Layout algorithms for node positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    /// Hierarchical top-down layout
    Hierarchical,
    /// Force-directed physics simulation
    ForceDirected,
    /// Circular arrangement
    Circular,
    /// Grid-based layout
    Grid,
    /// Custom tier-based layout
    TierBased,
}

/// Color schemes for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Default colors
    Default,
    /// Dark theme
    Dark,
    /// Blue enterprise theme
    Blue,
    /// High contrast
    HighContrast,
    /// Colorblind friendly
    ColorblindFriendly,
}

/// Network topology visualizer
pub struct TopologyVisualizer {
    config: VisualizationConfig,
}

/// Node visualization properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub tier: String,
    pub position: Option<(f64, f64)>,
    pub color: String,
    pub shape: NodeShape,
    pub size: NodeSize,
    pub metadata: HashMap<String, String>,
}

/// Connection visualization properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualConnection {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub connection_type: String,
    pub label: Option<String>,
    pub color: String,
    pub style: ConnectionStyle,
    pub weight: f64,
}

/// Node shapes for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeShape {
    Rectangle,
    Circle,
    Diamond,
    Hexagon,
    Cloud,
    Router,
    Switch,
    Server,
}

/// Node sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// Connection styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStyle {
    Solid,
    Dashed,
    Dotted,
    Bold,
    Wireless,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            format: VisualizationFormat::Svg,
            layout: LayoutAlgorithm::TierBased,
            color_scheme: ColorScheme::Default,
            show_labels: true,
            show_connection_details: true,
            group_by_tier: true,
        }
    }
}

impl TopologyVisualizer {
    /// Create a new topology visualizer
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Create visualizer with default configuration
    pub fn default() -> Self {
        Self::new(VisualizationConfig::default())
    }

    /// Convert SDN nodes to visual representation
    pub fn create_visual_nodes(&self, nodes: &[SDNNode]) -> Vec<VisualNode> {
        nodes.iter().map(|node| {
            let (color, shape, size) = self.get_node_appearance(&node.node_type, &node.tier);
            
            VisualNode {
                id: node.id.clone(),
                label: self.create_node_label(node),
                node_type: node.node_type.clone(),
                tier: node.tier.clone(),
                position: None, // Will be calculated by layout algorithm
                color,
                shape,
                size,
                metadata: node.metadata.clone(),
            }
        }).collect()
    }

    /// Convert SDN connections to visual representation
    pub fn create_visual_connections(&self, connections: &[SDNConnection]) -> Vec<VisualConnection> {
        connections.iter().map(|conn| {
            let (color, style, weight) = self.get_connection_appearance(&conn.connection_type);
            
            VisualConnection {
                id: conn.id.clone(),
                from_node: conn.from_node.clone(),
                to_node: conn.to_node.clone(),
                connection_type: conn.connection_type.clone(),
                label: if self.config.show_connection_details {
                    Some(self.create_connection_label(conn))
                } else {
                    None
                },
                color,
                style,
                weight,
            }
        }).collect()
    }

    /// Get visual appearance for a node based on type and tier
    fn get_node_appearance(&self, node_type: &str, tier: &str) -> (String, NodeShape, NodeSize) {
        let color = match (&self.config.color_scheme, node_type) {
            (ColorScheme::Default, "gateway") => "#FF6B6B".to_string(),
            (ColorScheme::Default, "switch") => "#4ECDC4".to_string(),
            (ColorScheme::Default, "server") => "#45B7D1".to_string(),
            (ColorScheme::Default, "workstation") => "#96CEB4".to_string(),
            (ColorScheme::Blue, _) => "#2E86AB".to_string(),
            (ColorScheme::Dark, _) => "#F8F9FA".to_string(),
            _ => "#6C757D".to_string(),
        };

        let shape = match node_type {
            "gateway" => NodeShape::Diamond,
            "switch" => NodeShape::Hexagon,
            "server" => NodeShape::Rectangle,
            "workstation" => NodeShape::Circle,
            _ => NodeShape::Rectangle,
        };

        let size = match tier {
            "super-cluster" => NodeSize::ExtraLarge,
            "cluster" => NodeSize::Large,
            "leaf" => NodeSize::Medium,
            "client" => NodeSize::Small,
            _ => NodeSize::Medium,
        };

        (color, shape, size)
    }

    /// Get visual appearance for a connection based on type
    fn get_connection_appearance(&self, connection_type: &str) -> (String, ConnectionStyle, f64) {
        let color = match (&self.config.color_scheme, connection_type) {
            (ColorScheme::Default, "ethernet") => "#2C3E50".to_string(),
            (ColorScheme::Default, "wireless") => "#E74C3C".to_string(),
            (ColorScheme::Default, "wan") => "#8E44AD".to_string(),
            _ => "#34495E".to_string(),
        };

        let style = match connection_type {
            "wireless" => ConnectionStyle::Wireless,
            "wan" => ConnectionStyle::Bold,
            "backup" => ConnectionStyle::Dashed,
            _ => ConnectionStyle::Solid,
        };

        let weight = match connection_type {
            "wan" => 3.0,
            "ethernet" => 2.0,
            "wireless" => 1.5,
            _ => 1.0,
        };

        (color, style, weight)
    }

    /// Create a label for a node
    fn create_node_label(&self, node: &SDNNode) -> String {
        if self.config.show_labels {
            format!("{}\n({})", node.id, node.node_type)
        } else {
            node.id.clone()
        }
    }

    /// Create a label for a connection
    fn create_connection_label(&self, connection: &SDNConnection) -> String {
        let mut parts = vec![connection.connection_type.clone()];
        
        if let Some(bandwidth) = connection.properties.get("bandwidth") {
            parts.push(bandwidth.clone());
        }
        
        if let Some(vlan) = connection.properties.get("vlan") {
            parts.push(format!("VLAN {}", vlan));
        }
        
        parts.join("\n")
    }

    /// Generate ASCII art representation of the topology
    pub fn generate_ascii_diagram(&self, nodes: &[VisualNode], connections: &[VisualConnection]) -> String {
        let mut diagram = String::new();
        
        // Group nodes by tier for ASCII layout
        let mut tiers: HashMap<String, Vec<&VisualNode>> = HashMap::new();
        for node in nodes {
            tiers.entry(node.tier.clone()).or_insert_with(Vec::new).push(node);
        }
        
        // Define tier order
        let tier_order = vec!["super-cluster", "cluster", "leaf", "client"];
        
        for tier in tier_order {
            if let Some(tier_nodes) = tiers.get(tier) {
                diagram.push_str(&format!("\n{} TIER:\n", tier.to_uppercase()));
                diagram.push_str(&"=".repeat(50));
                diagram.push('\n');
                
                for node in tier_nodes {
                    let symbol = match node.node_type.as_str() {
                        "gateway" => "◆",
                        "switch" => "⬢",
                        "server" => "▬",
                        "workstation" => "○",
                        _ => "□",
                    };
                    
                    diagram.push_str(&format!("{} {} ({})\n", symbol, node.label, node.node_type));
                }
            }
        }
        
        // Add connections section
        if !connections.is_empty() {
            diagram.push_str("\nCONNECTIONS:\n");
            diagram.push_str(&"=".repeat(50));
            diagram.push('\n');
            
            for conn in connections {
                let arrow = match conn.style {
                    ConnectionStyle::Wireless => "~~~>",
                    ConnectionStyle::Bold => "====>",
                    ConnectionStyle::Dashed => "---->",
                    _ => "----->",
                };
                
                let label = conn.label.as_ref().map(|l| format!(" [{}]", l)).unwrap_or_default();
                diagram.push_str(&format!("{} {} {}{}\n", 
                    conn.from_node, arrow, conn.to_node, label));
            }
        }
        
        diagram
    }

    /// Generate Mermaid diagram format
    pub fn generate_mermaid_diagram(&self, nodes: &[VisualNode], connections: &[VisualConnection]) -> String {
        let mut diagram = String::from("graph TD\n");
        
        // Add nodes
        for node in nodes {
            let shape_start = match node.shape {
                NodeShape::Circle => "((",
                NodeShape::Rectangle => "[",
                NodeShape::Diamond => "{",
                NodeShape::Hexagon => "{{",
                _ => "[",
            };
            
            let shape_end = match node.shape {
                NodeShape::Circle => "))",
                NodeShape::Rectangle => "]",
                NodeShape::Diamond => "}",
                NodeShape::Hexagon => "}}",
                _ => "]",
            };
            
            diagram.push_str(&format!("    {}{}\"{}\"{}\n", 
                node.id, shape_start, node.label.replace('\n', "<br/>"), shape_end));
        }
        
        // Add connections
        for conn in connections {
            let arrow = match conn.style {
                ConnectionStyle::Dashed => "-.->",
                ConnectionStyle::Dotted => "-..->",
                ConnectionStyle::Bold => "==>",
                _ => "-->",
            };
            
            if let Some(label) = &conn.label {
                diagram.push_str(&format!("    {} {}|{}| {}\n", 
                    conn.from_node, arrow, label.replace('\n', " "), conn.to_node));
            } else {
                diagram.push_str(&format!("    {} {} {}\n", 
                    conn.from_node, arrow, conn.to_node));
            }
        }
        
        // Add styling
        diagram.push_str("\n    classDef gateway fill:#ff6b6b\n");
        diagram.push_str("    classDef switch fill:#4ecdc4\n");
        diagram.push_str("    classDef server fill:#45b7d1\n");
        diagram.push_str("    classDef workstation fill:#96ceb4\n");
        
        for node in nodes {
            diagram.push_str(&format!("    class {} {}\n", node.id, node.node_type));
        }
        
        diagram
    }

    /// Generate DOT format for Graphviz
    pub fn generate_dot_diagram(&self, nodes: &[VisualNode], connections: &[VisualConnection]) -> String {
        let mut dot = String::from("digraph NetworkTopology {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    bgcolor=\"white\";\n");
        dot.push_str("    node [fontname=\"Arial\", fontsize=10];\n");
        dot.push_str("    edge [fontname=\"Arial\", fontsize=8];\n\n");
        
        // Add tier-based subgraphs for better layout
        if self.config.group_by_tier {
            let mut tiers: HashMap<String, Vec<&VisualNode>> = HashMap::new();
            for node in nodes {
                tiers.entry(node.tier.clone()).or_insert_with(Vec::new).push(node);
            }
            
            for (tier, tier_nodes) in tiers {
                dot.push_str(&format!("    subgraph cluster_{} {{\n", tier));
                dot.push_str(&format!("        label=\"{} Tier\";\n", tier.to_uppercase()));
                dot.push_str("        color=lightgrey;\n");
                
                for node in tier_nodes {
                    let shape = match node.shape {
                        NodeShape::Circle => "circle",
                        NodeShape::Rectangle => "box",
                        NodeShape::Diamond => "diamond",
                        NodeShape::Hexagon => "hexagon",
                        _ => "box",
                    };
                    
                    dot.push_str(&format!("        {} [label=\"{}\", shape={}, fillcolor=\"{}\", style=\"filled\"];\n",
                        node.id, node.label.replace('\n', "\\n"), shape, node.color));
                }
                
                dot.push_str("    }\n\n");
            }
        } else {
            // Add nodes without grouping
            for node in nodes {
                let shape = match node.shape {
                    NodeShape::Circle => "circle",
                    NodeShape::Rectangle => "box", 
                    NodeShape::Diamond => "diamond",
                    NodeShape::Hexagon => "hexagon",
                    _ => "box",
                };
                
                dot.push_str(&format!("    {} [label=\"{}\", shape={}, fillcolor=\"{}\", style=\"filled\"];\n",
                    node.id, node.label.replace('\n', "\\n"), shape, node.color));
            }
        }
        
        // Add connections
        for conn in connections {
            let style = match conn.style {
                ConnectionStyle::Dashed => "dashed",
                ConnectionStyle::Dotted => "dotted",
                ConnectionStyle::Bold => "bold",
                _ => "solid",
            };
            
            if let Some(label) = &conn.label {
                dot.push_str(&format!("    {} -> {} [label=\"{}\", color=\"{}\", style={}, penwidth={}];\n",
                    conn.from_node, conn.to_node, label.replace('\n', "\\n"), 
                    conn.color, style, conn.weight));
            } else {
                dot.push_str(&format!("    {} -> {} [color=\"{}\", style={}, penwidth={}];\n",
                    conn.from_node, conn.to_node, conn.color, style, conn.weight));
            }
        }
        
        dot.push_str("}\n");
        dot
    }
}

impl std::fmt::Debug for TopologyVisualizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TopologyVisualizer")
            .field("format", &self.config.format)
            .field("layout", &self.config.layout)
            .field("color_scheme", &self.config.color_scheme)
            .finish()
    }
}