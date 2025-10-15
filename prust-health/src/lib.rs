mod generated;

pub use generated::{HealthCheckRequest, HealthCheckResponse};
pub use generated::health_check_response::ServingStatus;

#[cfg(feature = "client")]
pub use generated::health_client::*;

#[cfg(feature = "server")]
pub use generated::health_server::*;
