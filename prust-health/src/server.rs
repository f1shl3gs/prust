use std::collections::HashMap;
use std::sync::Arc;

use prust::tonic::async_trait;
use prust::tonic::{Request, Response, Status};
use tokio::sync::{RwLock, watch};

use crate::{Health, HealthCheckRequest, HealthCheckResponse, HealthServer, ServingStatus};

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

#[async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let service_name = request.get_ref().service.as_str();
        let Some(status) = self.service_health(service_name).await else {
            return Err(Status::not_found("service not registered"));
        };

        Ok(Response::new(HealthCheckResponse { status }))
    }

    type WatchStream = WatchStream;

    async fn watch(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let service_name = request.get_ref().service.as_str();
        let status_rx = match self.statuses.read().await.get(service_name) {
            Some((_tx, rx)) => rx.clone(),
            None => return Err(Status::not_found("service not registered")),
        };

        Ok(Response::new(WatchStream::new(status_rx)))
    }
}

/// A watch stream for the health service.
pub struct WatchStream {
    inner: tokio_stream::wrappers::WatchStream<ServingStatus>,
}

impl WatchStream {
    fn new(status_rx: watch::Receiver<ServingStatus>) -> Self {
        let inner = tokio_stream::wrappers::WatchStream::new(status_rx);
        Self { inner }
    }
}

impl tokio_stream::Stream for WatchStream {
    type Item = Result<HealthCheckResponse, Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner)
            .poll_next(cx)
            .map(|opt| opt.map(|status| Ok(HealthCheckResponse { status })))
    }
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
        statuses: Arc::new(RwLock::new(HashMap::from([server_status]))),
    };

    let service = HealthService {
        statuses: Arc::clone(&reporter.statuses),
    };

    (reporter, HealthServer::new(service))
}
