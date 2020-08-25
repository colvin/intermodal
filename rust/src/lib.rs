//! Data enveloping format.
//!
//! A common structure into which data is embedded that provides a manifest of the type or schema
//! of the data as well as other identifying or contextualizing metadata.

use std::collections::HashMap;

/// An elemental data structure consisting of only a manifest.
///
/// All types derived from the `intermodal` scheme should be able to be deserialized into an
/// `Header`. Generic data handlers can expect to deserialize messages into this type and use the
/// manifest to determine a more precise type for the message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub manifest: Manifest,
}

impl<T> From<Packet<T>> for Header {
    fn from(data: Packet<T>) -> Self {
        Self {
            manifest: data.manifest,
        }
    }
}

impl From<Manifest> for Header {
    fn from(manifest: Manifest) -> Self {
        Self { manifest: manifest }
    }
}

/// Metadata that describes the content.
///
/// The manifest is used by application code that routes, stores, and processes data. It informs
/// those processes about what type of data it is and what schema it implements, the identity of
/// its origination point, and when it was sourced. It also carries arbitrary additional context as
/// a set of key-value pair strings.
///
/// ## Example
/// An example manifest in a variety of encoding formats.
///
/// ### YAML
/// ```yaml
/// manifest:
///   domain: example.org
///   scope: metrics/applications/some-app
///   kind: useractions
///   version: 2
///   origin: some-app-03.example.org
///   ctime: 2020-08-25T14:41:40Z
///   labels:
///     app-version: 2.3.1
/// ```
/// ### JSON
/// ```json
/// {
///   "manifest": {
///     "domain": "example.org",
///     "scope": "metrics/applications/some-app",
///     "kind": "useractions",
///     "version": 2,
///     "origin": "some-app-03.example.org",
///     "ctime": "2020-08-25T14:41:40Z",
///     "labels": {
///       "app-version": "2.3.1"
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    /// A DNS name identifying the organization that defines the type's schema.
    pub domain: String,

    /// An arbitrary string acting as a namespace element, conventionally formatted as a path.
    pub scope: String,

    /// The name of the type.
    pub kind: String,

    /// The version of the type's schema.
    pub version: u64,

    /// The identity of the source of this content.
    pub origin: String,

    /// The UTC timestamp at which the message was created.
    ///
    /// This is not necessarily the timestamp at which the data was sourced, collected, or
    /// otherwise obtained. Types requiring that degree of precision are responsible for conveying
    /// that information themselves.
    pub ctime: DateTime,

    /// Arbitrary key-value string pairs that provide additional context. Optional.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

impl<T> From<Packet<T>> for Manifest {
    fn from(data: Packet<T>) -> Self {
        data.manifest
    }
}

/// A complete data structure consisting of a manifest and the content.
///
/// All types derived from the `intermodal` scheme should be deserializable into a `Packet`,
/// once the specific derivative type has been determined.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Packet<T> {
    pub manifest: Manifest,
    pub content: T,
}

impl<T> Packet<T> {
    /// Create a `Packet` from an `Header` and some content.
    pub fn from_obj(obj: Header, content: T) -> Self {
        Packet {
            manifest: obj.manifest,
            content: content,
        }
    }
}

/// An alias for the underlying `DateTime` type from `chrono`.
pub type DateTime = chrono::DateTime<chrono::Utc>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct CpuMetrics {
        begin: DateTime,
        end: DateTime,
        interval_seconds: u16,
        idle_percent: Vec<u8>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct NetstatConnections {
        connections: Vec<NetstatConnection>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct NetstatConnection {
        local_addr: Ipv4Addr,
        local_port: u16,
        remote_addr: Option<Ipv4Addr>,
        remote_port: Option<u16>,
        state: TcpState,
        pid: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    enum TcpState {
        #[serde(rename = "LISTEN")]
        Listen,
        #[serde(rename = "ESTABLISHED")]
        Established,
        // ... omitting the rest
    }

    impl std::fmt::Display for TcpState {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Listen => "LISTEN",
                    Self::Established => "ESTABLISHED",
                }
            )
        }
    }

    #[test]
    fn test_json_cpu_metrics() {
        let mut filepath = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        filepath.push("../tests/cpu.metrics.example.org.json");
        let blob = std::fs::read_to_string(filepath).unwrap();

        assert!(
            serde_json::from_str::<Header>(&blob).is_ok(),
            "deserializable to Header"
        );

        assert!(
            serde_json::from_str::<Packet<CpuMetrics>>(&blob).is_ok(),
            "deserializable to Packet<CpuMetrics>",
        );

        let cpu_obj: Packet<CpuMetrics> = serde_json::from_str(&blob).unwrap();

        assert_eq!(cpu_obj.manifest.domain, "example.org");
        assert_eq!(cpu_obj.manifest.scope, "metrics");
        assert_eq!(cpu_obj.manifest.kind, "cpu");
        assert_eq!(cpu_obj.manifest.version, 1);
        assert_eq!(cpu_obj.manifest.labels.get("foo").unwrap(), "bar");
        assert_eq!(cpu_obj.content.interval_seconds, 10);
        assert_eq!(cpu_obj.content.idle_percent.len(), 6);
        assert_eq!(cpu_obj.content.idle_percent[2], 85);

        let wrong_obj = serde_json::from_str::<Packet<String>>(&blob);
        assert!(wrong_obj.is_err(), "not convertable to incompatible Packet");

        // From/Into works Packet --> to Header
        let _ = Header::from(cpu_obj.clone());
        let obj: Header = cpu_obj.clone().into();

        // Packet::from_obj()
        let cpu_obj_2: Packet<CpuMetrics> = Packet::from_obj(obj.clone(), cpu_obj.content.clone());
        assert_eq!(cpu_obj_2.manifest.ctime, cpu_obj.manifest.ctime);
        assert_eq!(cpu_obj_2.content.idle_percent, cpu_obj.content.idle_percent);
    }

    #[test]
    fn test_netstat_connections() {
        let mut jfilepath = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        jfilepath.push("../tests/netstat.connections.example.org.json");
        let jblob = std::fs::read_to_string(jfilepath).unwrap();

        assert!(
            serde_json::from_str::<Header>(&jblob).is_ok(),
            "json deserializable to Header"
        );

        assert!(
            serde_json::from_str::<Packet<NetstatConnections>>(&jblob).is_ok(),
            "json deserializable to Packet<NetstatConnections>"
        );

        let netstat: Packet<NetstatConnections> = serde_json::from_str(&jblob).unwrap();

        assert_eq!(netstat.manifest.scope, "connections");
        assert_eq!(netstat.manifest.kind, "netstat");
        assert_eq!(netstat.content.connections.len(), 2);
        assert_eq!(
            netstat.content.connections[0].local_addr,
            Ipv4Addr::from_str("127.0.0.1").unwrap(),
        );
        assert!(netstat.content.connections[0].remote_addr.is_none());
        assert_eq!(netstat.content.connections[0].state, TcpState::Listen);

        // From/Into
        let _ = Header::from(netstat.clone());
        let obj: Header = netstat.clone().into();

        let netstat_2: Packet<NetstatConnections> =
            Packet::from_obj(obj.clone(), netstat.content.clone());
        assert_eq!(netstat_2.manifest.ctime, netstat.manifest.ctime);
        assert_eq!(
            netstat_2.content.connections.len(),
            netstat.content.connections.len()
        );

        let mut yfilepath = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        yfilepath.push("../tests/netstat.connections.example.org.yaml");
        let yblob = std::fs::read_to_string(yfilepath).unwrap();

        assert!(
            serde_yaml::from_str::<Header>(&yblob).is_ok(),
            "yaml deserializable to Header"
        );

        assert!(
            serde_yaml::from_str::<Packet<NetstatConnections>>(&yblob).is_ok(),
            "yaml deserializable to Packet<NetstatConnections>"
        );

        let netstat_y: Packet<NetstatConnections> = serde_yaml::from_str(&yblob).unwrap();
        assert_eq!(netstat_y.manifest.ctime, netstat.manifest.ctime);
        assert_eq!(
            netstat_y.content.connections.len(),
            netstat.content.connections.len()
        );
        for i in 0..netstat_y.content.connections.len() {
            assert_eq!(
                netstat_y.content.connections[i].local_addr,
                netstat.content.connections[i].local_addr
            );
            assert_eq!(
                netstat_y.content.connections[i].local_port,
                netstat.content.connections[i].local_port
            );
            assert_eq!(
                netstat_y.content.connections[i].remote_addr,
                netstat.content.connections[i].remote_addr
            );
            assert_eq!(
                netstat_y.content.connections[i].remote_port,
                netstat.content.connections[i].remote_port
            );
            assert_eq!(
                netstat_y.content.connections[i].state,
                netstat.content.connections[i].state
            );
            assert_eq!(
                netstat_y.content.connections[i].pid,
                netstat.content.connections[i].pid
            );
        }
    }
}
