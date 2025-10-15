mod generated;
#[cfg(feature = "server")]
pub mod server;

pub use generated::health_check_response::ServingStatus;
pub use generated::{HealthCheckRequest, HealthCheckResponse};

#[cfg(feature = "client")]
pub use generated::health_client::*;

#[cfg(feature = "server")]
pub use generated::health_server::*;
