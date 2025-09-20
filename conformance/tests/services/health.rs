use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use health::health_check_response::ServingStatus;
use health::health_server::HealthServer;
use health::{HealthCheckRequest, HealthCheckResponse};
use tonic::codegen::tokio_stream;
use tonic::codegen::tokio_stream::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod health {
    include!("prust/grpc_health_v1.rs");
}

#[derive(Default)]
pub struct CustomServer {}

pub struct WatchStream {
    inner: tokio_stream::wrappers::WatchStream<ServingStatus>,
}

impl Stream for WatchStream {
    type Item = Result<HealthCheckResponse, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx).map(|n| {
            Some(Ok(HealthCheckResponse {
                status: n.unwrap_or_default(),
            }))
        })
    }
}

#[tonic::async_trait]
impl health::health_server::Health for CustomServer {
    async fn check(
        &self,
        req: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        assert_eq!(req.get_ref().service, "foo");

        let resp = Response::new(HealthCheckResponse {
            status: ServingStatus::Serving,
        });

        Ok(resp)
    }

    type WatchStream = WatchStream;

    async fn watch(
        &self,
        _req: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        todo!()
    }
}

#[tokio::test]
async fn hello() {
    let svr = HealthServer::new(CustomServer::default());
    let addr = "127.0.0.1:50051".parse().unwrap();

    tokio::spawn(async move { Server::builder().add_service(svr).serve(addr).await });

    tokio::time::sleep(Duration::from_secs(1)).await;

    let conn = tonic::transport::Endpoint::new("http://127.0.0.1:50051")
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = health::health_client::HealthClient::new(conn);
    let resp = client
        .check(Request::new(HealthCheckRequest {
            service: "foo".to_string(),
        }))
        .await
        .unwrap();

    println!("{:?}", resp.get_ref().status);
}
