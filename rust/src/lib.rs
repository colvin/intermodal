use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Object {
    pub meta: Meta,
}

impl<T> From<DataObject<T>> for Object {
    fn from(data: DataObject<T>) -> Object {
        Object { meta: data.meta }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Meta {
    pub domain: String,
    pub scope: String,
    pub kind: String,
    pub version: u64,
    pub origin: String,
    pub ctime: DateTime,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataObject<T> {
    pub meta: Meta,
    pub data: T,
}

impl<T> DataObject<T> {
    pub fn from_obj(obj: Object, data: T) -> Self {
        DataObject {
            meta: obj.meta,
            data: data,
        }
    }
}

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
            serde_json::from_str::<Object>(&blob).is_ok(),
            "deserializable to Object"
        );

        assert!(
            serde_json::from_str::<DataObject<CpuMetrics>>(&blob).is_ok(),
            "deserializable to DataObject<CpuMetrics>",
        );

        let cpu_obj: DataObject<CpuMetrics> = serde_json::from_str(&blob).unwrap();

        assert_eq!(cpu_obj.meta.domain, "example.org");
        assert_eq!(cpu_obj.meta.scope, "metrics");
        assert_eq!(cpu_obj.meta.kind, "cpu");
        assert_eq!(cpu_obj.meta.version, 1);
        assert_eq!(cpu_obj.meta.labels.get("foo").unwrap(), "bar");
        assert_eq!(cpu_obj.data.interval_seconds, 10);
        assert_eq!(cpu_obj.data.idle_percent.len(), 6);
        assert_eq!(cpu_obj.data.idle_percent[2], 85);

        let wrong_obj = serde_json::from_str::<DataObject<String>>(&blob);
        assert!(
            wrong_obj.is_err(),
            "not convertable to incompatible DataObject"
        );

        // From/Into works DataObject --> to Object
        let _ = Object::from(cpu_obj.clone());
        let obj: Object = cpu_obj.clone().into();

        // DataObject::from_obj()
        let cpu_obj_2: DataObject<CpuMetrics> =
            DataObject::from_obj(obj.clone(), cpu_obj.data.clone());
        assert_eq!(cpu_obj_2.meta.ctime, cpu_obj.meta.ctime);
        assert_eq!(cpu_obj_2.data.idle_percent, cpu_obj.data.idle_percent);
    }

    #[test]
    fn test_netstat_connections() {
        let mut jfilepath = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        jfilepath.push("../tests/netstat.connections.example.org.json");
        let jblob = std::fs::read_to_string(jfilepath).unwrap();

        assert!(
            serde_json::from_str::<Object>(&jblob).is_ok(),
            "json deserializable to Object"
        );

        assert!(
            serde_json::from_str::<DataObject<NetstatConnections>>(&jblob).is_ok(),
            "json deserializable to DataObject<NetstatConnections>"
        );

        let netstat: DataObject<NetstatConnections> = serde_json::from_str(&jblob).unwrap();

        assert_eq!(netstat.meta.scope, "connections");
        assert_eq!(netstat.meta.kind, "netstat");
        assert_eq!(netstat.data.connections.len(), 2);
        assert_eq!(
            netstat.data.connections[0].local_addr,
            Ipv4Addr::from_str("127.0.0.1").unwrap(),
        );
        assert!(netstat.data.connections[0].remote_addr.is_none());
        assert_eq!(netstat.data.connections[0].state, TcpState::Listen);

        // From/Into
        let _ = Object::from(netstat.clone());
        let obj: Object = netstat.clone().into();

        let netstat_2: DataObject<NetstatConnections> =
            DataObject::from_obj(obj.clone(), netstat.data.clone());
        assert_eq!(netstat_2.meta.ctime, netstat.meta.ctime);
        assert_eq!(
            netstat_2.data.connections.len(),
            netstat.data.connections.len()
        );

        let mut yfilepath = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        yfilepath.push("../tests/netstat.connections.example.org.yaml");
        let yblob = std::fs::read_to_string(yfilepath).unwrap();

        assert!(
            serde_yaml::from_str::<Object>(&yblob).is_ok(),
            "yaml deserializable to Object"
        );

        assert!(
            serde_yaml::from_str::<DataObject<NetstatConnections>>(&yblob).is_ok(),
            "yaml deserializable to DataObject<NetstatConnections>"
        );

        let netstat_y: DataObject<NetstatConnections> = serde_yaml::from_str(&yblob).unwrap();
        assert_eq!(netstat_y.meta.ctime, netstat.meta.ctime);
        assert_eq!(
            netstat_y.data.connections.len(),
            netstat.data.connections.len()
        );
        for i in 0..netstat_y.data.connections.len() {
            assert_eq!(
                netstat_y.data.connections[i].local_addr,
                netstat.data.connections[i].local_addr
            );
            assert_eq!(
                netstat_y.data.connections[i].local_port,
                netstat.data.connections[i].local_port
            );
            assert_eq!(
                netstat_y.data.connections[i].remote_addr,
                netstat.data.connections[i].remote_addr
            );
            assert_eq!(
                netstat_y.data.connections[i].remote_port,
                netstat.data.connections[i].remote_port
            );
            assert_eq!(
                netstat_y.data.connections[i].state,
                netstat.data.connections[i].state
            );
            assert_eq!(
                netstat_y.data.connections[i].pid,
                netstat.data.connections[i].pid
            );
        }
    }
}
