# Rusty Marzban Metrics Exporter

**Rusty Marzban Metrics Exporter** is a lightweight Rust-based application designed to fetch and expose system metrics from [Marzban](https://github.com/Gozargah/Marzban) for monitoring through Prometheus.

## Features

- Fetches system, node, core, and user data from Marzban API.
- Exposes data as Prometheus-compatible metrics for easy integration.
- Periodic updates of metrics for real-time tracking.
- Gracefully handles token-based authentication and automatic token renewal.
- Customizable configuration using environment variables.

## Requirements

- **Rust** (version 1.80 or later)

## Installation

### Pre-built binaries

Check out the [releases](https://github.com/like-a-freedom/rusty_marzban_metrics_exporter/releases) on GitHub for Linux x86/64 and ARM and also docker image.

### Build and Run

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/like-a-freedom/rusty_marzban_metrics_exporter.git
cd rusty_marzban_metrics_exporter
```

Build the project:

```bash
cargo build --release
```

You can configure the exporter using environment variables. The application supports the following variables, which can be defined in a `.env` file (sample in `src` directory) or passed directly when running:

- `MARZBAN_API_URL`: The base URL of your Marzban instance (e.g., `https://example.com/api`).
- `MARZBAN_API_KEY`: The API key to authenticate requests.
- `EXPORTER_PORT`: The port on which the metrics are exposed (default: `8080`).

Example `.env` file:

```ini
MARZBAN_API_URL=https://your-marzban-instance/api
MARZBAN_API_KEY=your-api-key
EXPORTER_PORT=8080
```

`.env` file must be placed in the same directory as binary.

Run the project:

```bash
cargo run
```


The exporter will now start fetching data from the Marzban API and exposing the metrics on a configurable port (default: `0.0.0.0:8000`).


### Docker

You can also run the exporter using Docker. Ensure that Docker is installed on your system, then build and run the container:

```bash
docker build -t rusty_marzban_metrics_exporter .
docker run --env-file .env -p 8080:8080 rusty_marzban_metrics_exporter
```
You can pull images from
- Github: `docker pull ghcr.io/like-a-freedom/rusty_marzban_metrics_exporter:latest`
- Docker Hub: `docker push expl0it99/rusty_marzban_metrics_exporter:latest`

Also check `docker-compose.yaml` in the repository.

## Usage

After running the exporter, you can view the exposed metrics at `http://ip:port/metrics`. Add this endpoint to your Prometheus configuration to start scraping data.

Sample Prometheus configuration:

```yaml
scrape_configs:
  - job_name: 'rusty_marzban_metrics_exporter'
    static_configs:
      - targets: ['ip:port']
```

## Metrics

The following metrics are currently supported:

- `marzban_system_data`: Provides information on the system.
- `marzban_node_data`: Node metrics such as uptime, status, etc.
- `marzban_core_data`: Core-related data including performance stats.
- `marzban_user_data`: Metrics about the users in the system.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues, submit pull requests, or suggest new features.
