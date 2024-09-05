use crate::api::{CoreData, Node, NodeUsageResponse, SystemData, UserResponse};
use prometheus::{Encoder, GaugeVec, Registry, TextEncoder};

// Export Registry to be used across the application
pub fn setup_metrics_registry() -> Registry {
    Registry::new()
}

pub struct Metrics {
    pub node_usage_coefficient_gauge: GaugeVec,
    pub node_uplink_gauge: GaugeVec,
    pub node_downlink_gauge: GaugeVec,
    pub node_version_info: GaugeVec,
    pub system_mem_total: GaugeVec,
    pub system_mem_used: GaugeVec,
    pub system_cpu_cores: GaugeVec,
    pub system_cpu_usage: GaugeVec,
    pub system_total_user: GaugeVec,
    pub system_users_active: GaugeVec,
    pub system_incoming_bandwidth: GaugeVec,
    pub system_outgoing_bandwidth: GaugeVec,
    pub system_incoming_bandwidth_speed: GaugeVec,
    pub system_outgoing_bandwidth_speed: GaugeVec,
    pub system_version_info: GaugeVec,
    pub core_started: GaugeVec,
    pub core_version_info: GaugeVec,
    pub user_used_traffic: GaugeVec,
}

pub fn create_metrics(registry: &Registry) -> Metrics {
    let node_usage_coefficient_gauge = create_gauge_vec(
        "node_usage_coefficient",
        "Node usage coefficient",
        &["node_name"],
        registry,
    );
    let node_uplink_gauge = create_gauge_vec(
        "node_uplink",
        "Node uplink bandwidth in bytes",
        &["node_name"],
        registry,
    );
    let node_downlink_gauge = create_gauge_vec(
        "node_downlink",
        "Node downlink bandwidth in bytes",
        &["node_name"],
        registry,
    );

    let node_version_info = create_gauge_vec(
        "node_version_info",
        "Node version information",
        &[
            "node_name",
            "xray_version",
            "status",
            "address",
            "port",
            "api_port",
        ],
        registry,
    );

    let system_mem_total =
        create_gauge_vec("system_mem_total", "Total memory in bytes", &[], registry);
    let system_mem_used =
        create_gauge_vec("system_mem_used", "Used memory in bytes", &[], registry);
    let system_cpu_cores =
        create_gauge_vec("system_cpu_cores", "Number of CPU cores", &[], registry);
    let system_cpu_usage =
        create_gauge_vec("system_cpu_usage", "CPU usage percentage", &[], registry);
    let system_total_user =
        create_gauge_vec("system_total_user", "Total number of users", &[], registry);
    let system_users_active = create_gauge_vec(
        "system_users_active",
        "Number of active users",
        &[],
        registry,
    );
    let system_incoming_bandwidth = create_gauge_vec(
        "system_incoming_bandwidth",
        "Incoming bandwidth in bytes",
        &[],
        registry,
    );
    let system_outgoing_bandwidth = create_gauge_vec(
        "system_outgoing_bandwidth",
        "Outgoing bandwidth in bytes",
        &[],
        registry,
    );
    let system_incoming_bandwidth_speed = create_gauge_vec(
        "system_incoming_bandwidth_speed",
        "Incoming bandwidth speed in bytes per second",
        &[],
        registry,
    );
    let system_outgoing_bandwidth_speed = create_gauge_vec(
        "system_outgoing_bandwidth_speed",
        "Outgoing bandwidth speed in bytes per second",
        &[],
        registry,
    );
    let system_version_info = create_gauge_vec(
        "system_version_info",
        "System version information",
        &["version"],
        registry,
    );

    let core_started = create_gauge_vec("core_started", "Core started status", &[], registry);
    let core_version_info = create_gauge_vec(
        "core_version_info",
        "Core version information",
        &["version"],
        registry,
    );
    let user_used_traffic = create_gauge_vec(
        "user_used_traffic",
        "User used traffic in bytes",
        &["username", "status"],
        registry,
    );

    Metrics {
        node_usage_coefficient_gauge,
        node_uplink_gauge,
        node_downlink_gauge,
        node_version_info,
        system_mem_total,
        system_mem_used,
        system_cpu_cores,
        system_cpu_usage,
        system_total_user,
        system_users_active,
        system_incoming_bandwidth,
        system_outgoing_bandwidth,
        system_incoming_bandwidth_speed,
        system_outgoing_bandwidth_speed,
        system_version_info,
        core_started,
        core_version_info,
        user_used_traffic,
    }
}

fn create_gauge_vec(name: &str, help: &str, labels: &[&str], registry: &Registry) -> GaugeVec {
    let gauge_vec = GaugeVec::new(prometheus::Opts::new(name, help), labels)
        .unwrap_or_else(|err| panic!("Error creating gauge: {}", err));
    registry
        .register(Box::new(gauge_vec.clone()))
        .unwrap_or_else(|err| panic!("Error registering gauge: {}", err));
    gauge_vec
}

pub fn update_metrics(
    metrics: &Metrics,
    nodes: &[Node],
    node_usages: &NodeUsageResponse,
    system_data: &SystemData,
    core_data: &CoreData,
    users: &UserResponse,
) {
    // Update node metrics
    for node in nodes {
        metrics
            .node_usage_coefficient_gauge
            .with_label_values(&[&node.name])
            .set(node.usage_coefficient);

        metrics
            .node_version_info
            .with_label_values(&[
                &node.name,
                &node.xray_version,
                &node.status,
                &node.address,
                &node.port.to_string(),
                &node.api_port.to_string(),
            ])
            .set(1.0);
    }

    // Update node usage metrics
    for usage in &node_usages.usages {
        metrics
            .node_uplink_gauge
            .with_label_values(&[&usage.node_name])
            .set(usage.uplink as f64);
        metrics
            .node_downlink_gauge
            .with_label_values(&[&usage.node_name])
            .set(usage.downlink as f64);
    }

    // Update system metrics
    metrics
        .system_mem_total
        .with_label_values(&[])
        .set(system_data.mem_total as f64);
    metrics
        .system_mem_used
        .with_label_values(&[])
        .set(system_data.mem_used as f64);
    metrics
        .system_cpu_cores
        .with_label_values(&[])
        .set(system_data.cpu_cores as f64);
    metrics
        .system_cpu_usage
        .with_label_values(&[])
        .set(system_data.cpu_usage);
    metrics
        .system_total_user
        .with_label_values(&[])
        .set(system_data.total_user as f64);
    metrics
        .system_users_active
        .with_label_values(&[])
        .set(system_data.users_active as f64);
    metrics
        .system_incoming_bandwidth
        .with_label_values(&[])
        .set(system_data.incoming_bandwidth as f64);
    metrics
        .system_outgoing_bandwidth
        .with_label_values(&[])
        .set(system_data.outgoing_bandwidth as f64);
    metrics
        .system_incoming_bandwidth_speed
        .with_label_values(&[])
        .set(system_data.incoming_bandwidth_speed as f64);
    metrics
        .system_outgoing_bandwidth_speed
        .with_label_values(&[])
        .set(system_data.outgoing_bandwidth_speed as f64);
    metrics
        .system_version_info
        .with_label_values(&[&system_data.version])
        .set(1.0);

    // Update core metrics
    metrics
        .core_started
        .with_label_values(&[])
        .set(core_data.started as i64 as f64);
    metrics
        .core_version_info
        .with_label_values(&[&core_data.version])
        .set(1.0);

    // Update user metrics
    for user in &users.users {
        metrics
            .user_used_traffic
            .with_label_values(&[&user.username, &user.status])
            .set(user.used_traffic as f64);
    }
}

pub fn gather_metrics_output(registry: &Registry) -> String {
    let mut buffer = Vec::new();
    let metric_families = registry.gather();
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
