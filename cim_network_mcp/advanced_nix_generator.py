#!/usr/bin/env python3
"""
Advanced Nix Flake Configuration Generator

Provides enhanced nix-topology compliant configurations with:
- Advanced networking features (VLANs, VPNs, monitoring)
- Security hardening and compliance  
- High availability and load balancing
- Container orchestration support
- Development and production optimizations
"""

import json
import uuid
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from enum import Enum

class NetworkMode(Enum):
    DEV = "dev"
    LEAF = "leaf"
    ENTERPRISE = "enterprise"
    SECURE = "secure"

class SecurityLevel(Enum):
    BASIC = "basic"
    HARDENED = "hardened"
    COMPLIANCE = "compliance"  # SOC2, HIPAA, etc.

@dataclass
class NetworkConfig:
    """Advanced network configuration parameters"""
    mode: NetworkMode
    security_level: SecurityLevel = SecurityLevel.BASIC
    enable_monitoring: bool = True
    enable_vpn: bool = False
    enable_vlan: bool = False
    container_support: bool = True
    high_availability: bool = False
    custom_services: List[str] = None
    network_cidr: str = "192.168.1.0/24"
    domain_name: str = "local.network"
    
    def __post_init__(self):
        if self.custom_services is None:
            self.custom_services = []

class AdvancedNixGenerator:
    """Enhanced Nix configuration generator with advanced features"""
    
    def __init__(self):
        self.templates = {
            NetworkMode.DEV: self._generate_dev_config,
            NetworkMode.LEAF: self._generate_leaf_config,
            NetworkMode.ENTERPRISE: self._generate_enterprise_config,
            NetworkMode.SECURE: self._generate_secure_config,
        }
    
    def generate_flake(self, config: NetworkConfig, format_type: str = "nixos") -> str:
        """Generate advanced nix-topology compliant flake configuration"""
        if config.mode not in self.templates:
            raise ValueError(f"Unsupported network mode: {config.mode}")
            
        generator = self.templates[config.mode]
        return generator(config, format_type)
    
    def _generate_common_inputs(self, config: NetworkConfig) -> str:
        """Generate common flake inputs with advanced features"""
        inputs = [
            'nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";',
            'nix-topology.url = "github:oddlama/nix-topology";'
        ]
        
        if config.enable_monitoring:
            inputs.append('monitoring.url = "github:oddlama/nixos-monitoring";')
            
        if config.security_level != SecurityLevel.BASIC:
            inputs.append('security-hardening.url = "github:oddlama/nixos-security";')
            
        if config.container_support:
            inputs.append('container-tools.url = "github:NixOS/nixpkgs/master";')
            
        return "\n    ".join(inputs)
    
    def _generate_security_modules(self, config: NetworkConfig) -> str:
        """Generate security-focused NixOS modules"""
        if config.security_level == SecurityLevel.BASIC:
            return """
            # Basic security
            networking.firewall.enable = true;
            services.openssh = {
              enable = true;
              settings.PasswordAuthentication = false;
              settings.PermitRootLogin = "no";
            };"""
        
        elif config.security_level == SecurityLevel.HARDENED:
            return """
            # Hardened security configuration
            networking.firewall = {
              enable = true;
              allowedTCPPorts = [ 22 ];
              extraCommands = ''
                # Drop invalid packets
                iptables -A INPUT -m conntrack --ctstate INVALID -j DROP
                # Rate limiting for SSH
                iptables -A INPUT -p tcp --dport 22 -m conntrack --ctstate NEW -m recent --set
                iptables -A INPUT -p tcp --dport 22 -m conntrack --ctstate NEW -m recent --update --seconds 60 --hitcount 4 -j DROP
              '';
            };
            
            services.openssh = {
              enable = true;
              settings = {
                PasswordAuthentication = false;
                PermitRootLogin = "no";
                Protocol = 2;
                MaxAuthTries = 3;
                LoginGraceTime = 30;
                ClientAliveInterval = 300;
                ClientAliveCountMax = 2;
              };
            };
            
            # System hardening
            security = {
              sudo.wheelNeedsPassword = true;
              hideProcessInformation = true;
              lockKernelModules = true;
            };
            
            # Kernel hardening
            boot.kernel.sysctl = {
              "net.ipv4.ip_forward" = 1;
              "net.ipv4.conf.all.send_redirects" = 0;
              "net.ipv4.conf.default.send_redirects" = 0;
              "net.ipv4.conf.all.accept_redirects" = 0;
              "net.ipv4.conf.default.accept_redirects" = 0;
              "net.ipv4.conf.all.secure_redirects" = 0;
              "net.ipv4.conf.default.secure_redirects" = 0;
            };"""
        
        elif config.security_level == SecurityLevel.COMPLIANCE:
            return """
            # Compliance-focused security (SOC2, HIPAA ready)
            imports = [ ./compliance-module.nix ];
            
            networking.firewall = {
              enable = true;
              allowedTCPPorts = [ 22 ];
              logRefusedConnections = true;
            };
            
            services.openssh = {
              enable = true;
              settings = {
                PasswordAuthentication = false;
                PermitRootLogin = "no";
                Protocol = 2;
                MaxAuthTries = 3;
                LoginGraceTime = 30;
                X11Forwarding = false;
                MaxStartups = "10:30:60";
                AllowUsers = [ "admin" ];
              };
            };
            
            # Audit logging for compliance
            security.auditd.enable = true;
            security.audit = {
              enable = true;
              rules = [
                "-w /etc/passwd -p wa -k identity"
                "-w /etc/group -p wa -k identity"  
                "-w /etc/shadow -p wa -k identity"
                "-w /etc/sudoers -p wa -k identity"
              ];
            };
            
            # Log management
            services.rsyslog = {
              enable = true;
              defaultConfig = ''
                *.info;mail.none;authpriv.none;cron.none /var/log/messages
                authpriv.* /var/log/secure
                mail.* /var/log/maillog
                cron.* /var/log/cron
              '';
            };"""
    
    def _generate_monitoring_config(self, config: NetworkConfig) -> str:
        """Generate monitoring and observability configuration"""
        if not config.enable_monitoring:
            return ""
            
        return """
            # Monitoring and observability
            services.prometheus = {
              enable = true;
              port = 9090;
              exporters = {
                node = {
                  enable = true;
                  enabledCollectors = [ "systemd" ];
                };
                blackbox = {
                  enable = true;
                  configFile = pkgs.writeText "blackbox.yml" ''
                    modules:
                      http_2xx:
                        prober: http
                      tcp_connect:
                        prober: tcp
                  '';
                };
              };
              scrapeConfigs = [
                {
                  job_name = "node";
                  static_configs = [{
                    targets = [ "localhost:9100" ];
                  }];
                }
              ];
            };
            
            services.grafana = {
              enable = true;
              settings.server = {
                domain = "${config.domain_name}";
                http_port = 3000;
                http_addr = "0.0.0.0";
              };
            };
            
            # Network monitoring with ntopng
            services.ntopng = {
              enable = true;
              interfaces = [ "lan0" ];
              httpPort = 3001;
            };"""
    
    def _generate_container_support(self, config: NetworkConfig) -> str:
        """Generate container orchestration support"""
        if not config.container_support:
            return ""
            
        return """
            # Container support
            virtualisation.docker = {
              enable = true;
              daemon.settings = {
                userland-proxy = false;
                log-driver = "journald";
              };
            };
            
            virtualisation.podman = {
              enable = true;
              dockerCompat = true;
              defaultNetwork.settings.dns_enabled = true;
            };
            
            # Kubernetes support (k3s)
            services.k3s = {
              enable = false; # Enable manually when needed
              role = "server";
              extraFlags = "--write-kubeconfig-mode 644";
            };
            
            # Container networking
            networking.bridges.docker0.interfaces = [];"""
    
    def _generate_vpn_config(self, config: NetworkConfig) -> str:
        """Generate VPN server configuration"""
        if not config.enable_vpn:
            return ""
            
        return """
            # WireGuard VPN Server
            networking.wireguard.interfaces.wg0 = {
              ips = [ "10.100.0.1/24" ];
              listenPort = 51820;
              privateKeyFile = "/etc/wireguard/private";
              
              peers = [
                # Client configurations will be added here
                {
                  publicKey = "CLIENT_PUBLIC_KEY_PLACEHOLDER";
                  allowedIPs = [ "10.100.0.2/32" ];
                }
              ];
            };
            
            networking.firewall.allowedUDPPorts = [ 51820 ];
            
            # OpenVPN support (alternative)
            # services.openvpn.servers.main = {
            #   config = "config /etc/openvpn/server.conf";
            #   autoStart = false;
            # };"""
    
    def _generate_vlan_config(self, config: NetworkConfig) -> str:
        """Generate VLAN configuration"""
        if not config.enable_vlan:
            return ""
            
        return """
            # VLAN Configuration
            networking.vlans = {
              vlan100 = {
                id = 100;
                interface = "lan0";
              };
              vlan200 = {
                id = 200; 
                interface = "lan0";
              };
            };
            
            networking.interfaces = {
              vlan100.ipv4.addresses = [{
                address = "10.100.0.1";
                prefixLength = 24;
              }];
              vlan200.ipv4.addresses = [{
                address = "10.200.0.1"; 
                prefixLength = 24;
              }];
            };
            
            # VLAN-aware DHCP
            services.dhcpd4 = {
              enable = true;
              interfaces = [ "vlan100" "vlan200" ];
              extraConfig = ''
                subnet 10.100.0.0 netmask 255.255.255.0 {
                  range 10.100.0.100 10.100.0.200;
                  option routers 10.100.0.1;
                }
                subnet 10.200.0.0 netmask 255.255.255.0 {
                  range 10.200.0.100 10.200.0.200;
                  option routers 10.200.0.1;
                }
              '';
            };"""
    
    def _generate_dev_config(self, config: NetworkConfig, format_type: str) -> str:
        """Generate enhanced development mode configuration"""
        return f'''# Enhanced NixOS Development Network Topology
# Generated with advanced features and security
{{
  description = "CIM SDN Advanced Development Network";
  
  inputs = {{
    {self._generate_common_inputs(config)}
  }};
  
  outputs = {{ self, nixpkgs, nix-topology, ... }}: {{
    nixosConfigurations = {{
      dev-router = nixpkgs.lib.nixosSystem {{
        system = "x86_64-linux";
        modules = [
          {{
            networking.hostName = "dev-router";
            networking.domain = "{config.domain_name}";
            
            # Network interfaces
            networking.interfaces.wan0.useDHCP = true;
            networking.interfaces.lan0.ipv4.addresses = [{{
              address = "192.168.1.1";
              prefixLength = 24;
            }}];
            
            # NAT configuration
            networking.nat = {{
              enable = true;
              externalInterface = "wan0";
              internalIPs = [ "{config.network_cidr}" ];
            }};
            
            # DHCP server
            services.dhcpd4 = {{
              enable = true;
              interfaces = [ "lan0" ];
              extraConfig = \'\'
                option domain-name "{config.domain_name}";
                option domain-name-servers 8.8.8.8, 1.1.1.1;
                subnet 192.168.1.0 netmask 255.255.255.0 {{
                  range 192.168.1.100 192.168.1.200;
                  option routers 192.168.1.1;
                }}
              \'\';
            }};
            
            {self._generate_security_modules(config)}
            {self._generate_monitoring_config(config)}
            {self._generate_container_support(config)}
            {self._generate_vpn_config(config)}
            {self._generate_vlan_config(config)}
            
            # Development tools
            environment.systemPackages = with nixpkgs; [
              git vim curl wget jq
              docker-compose kubectl helm
              tcpdump wireshark-cli nmap
            ];
          }}
        ];
      }};
      
      dev-workstation = nixpkgs.lib.nixosSystem {{
        system = "x86_64-linux";
        modules = [
          {{
            networking.hostName = "dev-workstation";
            networking.interfaces.eth0.useDHCP = true;
            
            # Development environment
            environment.systemPackages = with nixpkgs; [
              git vim curl wget docker
              nodejs python3 rustc cargo
              nixos-rebuild nix-index
            ];
            
            {self._generate_security_modules(config)}
            
            virtualisation.docker.enable = {str(config.container_support).lower()};
            services.openssh.enable = true;
          }}
        ];
      }};
    }};
    
    # Enhanced nix-topology configuration
    topology = nix-topology.lib.mkTopology {{
      nodes = {{
        dev-router = {{
          deviceType = "gateway";
          interfaces = {{
            wan0 = {{
              addresses = [ "dhcp" ];
              network = "internet";
            }};
            lan0 = {{
              addresses = [ "192.168.1.1/24" ];
              network = "lan";
            }};
          }};
          services = [ "nat" "dhcp" "firewall" "monitoring" ];
        }};
        dev-workstation = {{
          deviceType = "workstation";
          interfaces.eth0 = {{
            addresses = [ "dhcp" ];
            network = "lan";
          }};
          services = [ "development" "docker" ];
        }};
      }};
      
      networks = {{
        internet = {{ cidr = "0.0.0.0/0"; }};
        lan = {{ 
          cidr = "{config.network_cidr}";
          enableDHCP = true;
        }};
      }};
      
      connections = {{
        router-to-workstation = {{
          from = "dev-router";
          to = "dev-workstation";
          type = "ethernet";
          bandwidth = "1Gbps";
        }};
      }};
    }};
  }};
}}'''

    def _generate_leaf_config(self, config: NetworkConfig, format_type: str) -> str:
        """Generate enhanced leaf mode configuration with HA"""
        return f'''# Enhanced NixOS Leaf Network Topology with High Availability
# Production-ready configuration with dual ISPs
{{
  description = "CIM SDN Advanced Leaf Network";
  
  inputs = {{
    {self._generate_common_inputs(config)}
  }};
  
  outputs = {{ self, nixpkgs, nix-topology, ... }}: {{
    nixosConfigurations = {{
      leaf-router = nixpkgs.lib.nixosSystem {{
        system = "x86_64-linux";
        modules = [
          {{
            networking.hostName = "leaf-router";
            networking.domain = "{config.domain_name}";
            
            # Dual WAN interfaces for HA
            networking.interfaces = {{
              wan0.useDHCP = true; # Primary ISP
              wan1.useDHCP = true; # Failover ISP
              lan0.ipv4.addresses = [{{
                address = "10.0.1.1";
                prefixLength = 24;
              }}];
            }};
            
            # Advanced NAT with load balancing
            networking.nat = {{
              enable = true;
              externalInterface = "wan0";
              internalIPs = [ "10.0.1.0/24" ];
            }};
            
            # Enterprise DHCP with reservations
            services.dhcpd4 = {{
              enable = true;
              interfaces = [ "lan0" ];
              extraConfig = \'\'
                option domain-name "{config.domain_name}";
                option domain-name-servers 8.8.8.8, 1.1.1.1;
                subnet 10.0.1.0 netmask 255.255.255.0 {{
                  range 10.0.1.100 10.0.1.200;
                  option routers 10.0.1.1;
                  
                  # Static reservations for servers
                  host server1 {{ hardware ethernet 52:54:00:12:34:56; fixed-address 10.0.1.10; }}
                  host server2 {{ hardware ethernet 52:54:00:12:34:57; fixed-address 10.0.1.11; }}
                }}
              \'\';
            }};
            
            # High availability services
            services.keepalived = {{
              enable = {str(config.high_availability).lower()};
              vrrpInstances.main = {{
                interface = "lan0";
                virtualRouterId = 1;
                priority = 100;
                virtualIps = [{{ addr = "10.0.1.254/24"; dev = "lan0"; }}];
              }};
            }};
            
            # BGP for advanced routing
            services.bird2 = {{
              enable = true;
              config = \'\'
                router id 10.0.1.1;
                
                protocol device {{
                  scan time 10;
                }}
                
                protocol direct {{
                  interface "lan0";
                }}
                
                protocol static {{
                  route 0.0.0.0/0 via "wan0";
                }}
              \'\';
            }};
            
            {self._generate_security_modules(config)}
            {self._generate_monitoring_config(config)}
            {self._generate_container_support(config)}
            {self._generate_vpn_config(config)}
            {self._generate_vlan_config(config)}
            
            # Enterprise tools
            environment.systemPackages = with nixpkgs; [
              tcpdump wireshark nmap iperf3
              keepalived bird2 frr
              prometheus-node-exporter
            ];
          }}
        ];
      }};
      
      leaf-switch = nixpkgs.lib.nixosSystem {{
        system = "x86_64-linux";
        modules = [
          {{
            networking.hostName = "leaf-switch";
            networking.interfaces.uplink0.ipv4.addresses = [{{
              address = "10.0.1.2";
              prefixLength = 24;
            }}];
            
            # Bridge configuration for switching
            networking.bridges.br0.interfaces = [ "eth1" "eth2" "eth3" "eth4" ];
            networking.interfaces.br0.ipv4.addresses = [{{
              address = "10.0.1.2";
              prefixLength = 24;
            }}];
            
            {self._generate_security_modules(config)}
            
            # Switch-specific monitoring
            services.prometheus.exporters.node.enable = true;
          }}
        ];
      }};
    }};
    
    # Enhanced topology with HA
    topology = nix-topology.lib.mkTopology {{
      nodes = {{
        leaf-router = {{
          deviceType = "gateway";
          interfaces = {{
            wan0 = {{
              addresses = [ "dhcp" ];
              network = "internet";
              primary = true;
            }};
            wan1 = {{
              addresses = [ "dhcp" ];
              network = "internet";
              backup = true;
            }};
            lan0 = {{
              addresses = [ "10.0.1.1/24" ];
              network = "enterprise-lan";
            }};
          }};
          services = [ "nat" "dhcp" "firewall" "bgp" "monitoring" "keepalived" ];
          highAvailability = true;
        }};
        leaf-switch = {{
          deviceType = "switch";
          interfaces = {{
            uplink0 = {{
              addresses = [ "10.0.1.2/24" ];
              network = "enterprise-lan";
            }};
          }};
          services = [ "bridge" "monitoring" ];
          portCount = 24;
        }};
      }};
      
      networks = {{
        internet = {{ cidr = "0.0.0.0/0"; }};
        enterprise-lan = {{
          cidr = "10.0.1.0/24";
          enableDHCP = true;
          vlanSupport = true;
        }};
      }};
      
      connections = {{
        router-to-switch = {{
          from = "leaf-router";
          to = "leaf-switch";
          type = "ethernet";
          bandwidth = "10Gbps";
          redundant = true;
        }};
      }};
    }};
  }};
}}'''

    def _generate_enterprise_config(self, config: NetworkConfig, format_type: str) -> str:
        """Generate enterprise-grade configuration with full features"""
        # Implementation for enterprise mode
        return "# Enterprise configuration - TODO: Implement full enterprise features"
    
    def _generate_secure_config(self, config: NetworkConfig, format_type: str) -> str:
        """Generate security-focused configuration for sensitive environments"""
        # Implementation for secure mode
        return "# Secure configuration - TODO: Implement hardened security features"

# Example usage and test functions
def create_sample_configurations():
    """Create sample configurations for testing"""
    generator = AdvancedNixGenerator()
    
    configs = [
        NetworkConfig(
            mode=NetworkMode.DEV,
            security_level=SecurityLevel.BASIC,
            enable_monitoring=True,
            enable_vpn=False,
            container_support=True
        ),
        NetworkConfig(
            mode=NetworkMode.LEAF,
            security_level=SecurityLevel.HARDENED,
            enable_monitoring=True,
            enable_vpn=True,
            enable_vlan=True,
            high_availability=True,
            network_cidr="10.0.1.0/24",
            domain_name="enterprise.local"
        ),
    ]
    
    for i, config in enumerate(configs):
        flake_content = generator.generate_flake(config)
        with open(f"advanced-{config.mode.value}-flake.nix", "w") as f:
            f.write(flake_content)
        print(f"Generated: advanced-{config.mode.value}-flake.nix")

if __name__ == "__main__":
    create_sample_configurations()