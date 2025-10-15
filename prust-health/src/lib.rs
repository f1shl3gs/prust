mod generated;

pub use generated::{HealthCheckRequest, HealthCheckResponse};
pub use generated::health_check_response::ServingStatus;

#[cfg(feature = "client")]
pub use generated::health_client::*;

#[cfg(feature = "server")]
pub use generated::health_server::*;

#[cfg(feature = "server")]
mod server {
    use std::collections::HashMap;
    use std::sync::{Arc};
    use tokio::sync::{watch, RwLock};
    use crate::{Health, HealthServer, ServingStatus};

    type StatusPair = (watch::Sender<ServingStatus>, watch::Receiver<ServingStatus>);

    /// A handle providing methods to update the health status of gRPC services. A
    /// `HealthReporter` is connected to a `HealthServer` which serves the statuses
    /// over the `grpc.health.v1.Health` service.
    #[derive(Clone, Debug)]
    pub struct HealthReporter {
        statuses: Arc<RwLock<HashMap<String, StatusPair>>>,
    }

    /// A service providing implementations of gRPC health checking protocol.
    #[derive(Debug)]
    pub struct HealthService {
        statuses: Arc<RwLock<HashMap<String, StatusPair>>>,
    }

    /// Creates a `HealthReporter` and a linked `HealthServer` pair. Together,
    /// these types can be used to serve the gRPC Health Checking service.
    ///
    /// A `HealthReporter` is used to update the state of gRPC services.
    ///
    /// A `HealthServer` is a Tonic gRPC server for the `grpc.health.v1.Health`,
    /// which can be added to a Tonic runtime using `add_service` on the runtime
    /// builder.
    pub fn health_reporter() -> (HealthReporter, HealthServer<impl Health>) {
        let server_status = ("".to_string(), watch::channel(ServingStatus::Serving));
        let reporter = HealthReporter {
            statuses: Arc::new(RwLock::new(HashMap::from([server_status])),),
        };

        let service = HealthService {
            statuses: Arc::clone(&reporter.statuses),
        };

        (reporter, HealthServer::new(service))
    }
}
