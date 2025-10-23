use prust::*;
#[derive(Debug, Default)]
pub struct HealthCheckRequest {
    pub service: String,
}
impl Deserialize for HealthCheckRequest {
    fn decode(src: &[u8]) -> Result<Self, DecodeError> {
        let mut buf = Reader::new(src);
        let mut msg: Self = Default::default();
        while buf.pos < buf.src.len() {
            let tag = buf.src[buf.pos] as u32;
            buf.pos += 1;
            match tag {
                10 => msg.service = buf.read_string()?,
                _ => {
                    buf.pos -= 1;
                    let tag = buf.read_uint32()?;
                    buf.read_unknown(tag)?;
                }
            }
        }
        Ok(msg)
    }
}
impl Serialize for HealthCheckRequest {
    fn encoded_len(&self) -> usize {
        if !self.service.is_empty() {
            1 + sizeof_len(self.service.len())
        } else {
            0
        }
    }
    fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut buf = Writer::new(buf);
        if !self.service.is_empty() {
            buf.write(10, self.service.as_str(), Writer::write_string)?
        }
        Ok(buf.pos)
    }
}
#[derive(Clone, Debug, Default)]
pub struct HealthCheckResponse {
    pub status: health_check_response::ServingStatus,
}
impl Deserialize for HealthCheckResponse {
    fn decode(src: &[u8]) -> Result<Self, DecodeError> {
        let mut buf = Reader::new(src);
        let mut msg: Self = Default::default();
        while buf.pos < buf.src.len() {
            let tag = buf.src[buf.pos] as u32;
            buf.pos += 1;
            match tag {
                8 => msg.status = buf.read_enum()?,
                _ => {
                    buf.pos -= 1;
                    let tag = buf.read_uint32()?;
                    buf.read_unknown(tag)?;
                }
            }
        }
        Ok(msg)
    }
}
impl Serialize for HealthCheckResponse {
    fn encoded_len(&self) -> usize {
        if self.status != health_check_response::ServingStatus::Unknown {
            1 + 1
        } else {
            0
        }
    }
    fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut buf = Writer::new(buf);
        if self.status != health_check_response::ServingStatus::Unknown {
            buf.write(8, self.status as i32, Writer::write_int32)?
        }
        Ok(buf.pos)
    }
}
pub mod health_check_response {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub enum ServingStatus {
        #[default]
        Unknown = 0,
        Serving = 1,
        NotServing = 2,
        ServiceUnknown = 3,
    }
    impl TryFrom<i32> for ServingStatus {
        type Error = DecodeError;
        fn try_from(value: i32) -> Result<Self, DecodeError> {
            match value {
                0 => Ok(ServingStatus::Unknown),
                1 => Ok(ServingStatus::Serving),
                2 => Ok(ServingStatus::NotServing),
                3 => Ok(ServingStatus::ServiceUnknown),
                _ => Err(DecodeError::UnknownVariant("ServingStatus", value)),
            }
        }
    }
}
#[cfg(feature = "client")]
pub mod health_client {
    #![allow(dead_code, unused_imports, unused_variables)]
    use super::*;
    use tonic::codegen::*;
    #[derive(Clone, Debug)]
    pub struct HealthClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl HealthClient<tonic::transport::Channel> {
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> HealthClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: http::Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> HealthClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                    >,
                >,
            <T as Service<http::Request<tonic::body::Body>>>::Error: Into<StdError> + Send + Sync,
        {
            HealthClient::new(InterceptedService::new(inner, interceptor))
        }
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn check(
            &mut self,
            req: impl tonic::IntoRequest<HealthCheckRequest>,
        ) -> Result<tonic::Response<HealthCheckResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|err| {
                tonic::Status::unknown(format!("Service was not ready: {}", err.into()))
            })?;
            let codec = prust::tonic_codec::Codec::default();
            let path = http::uri::PathAndQuery::from_static("/grpc.health.v1.Health/Check");
            let mut req = req.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc.health.v1.Health", "Check"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn watch(
            &mut self,
            req: impl tonic::IntoRequest<HealthCheckRequest>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<HealthCheckResponse>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|err| {
                tonic::Status::unknown(format!("Service was not ready: {}", err.into()))
            })?;
            let codec = prust::tonic_codec::Codec::default();
            let path = http::uri::PathAndQuery::from_static("/grpc.health.v1.Health/Watch");
            let mut req = req.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc.health.v1.Health", "Watch"));
            self.inner.server_streaming(req, path, codec).await
        }
    }
}
#[cfg(feature = "server")]
pub mod health_server {
    #![allow(dead_code, unused_imports)]
    use super::*;
    use tonic::codegen::*;
    #[async_trait]
    pub trait Health: Send + Sync + 'static {
        async fn check(
            &self,
            req: tonic::Request<HealthCheckRequest>,
        ) -> Result<tonic::Response<HealthCheckResponse>, tonic::Status>;
        type WatchStream: tokio_stream::Stream<Item = Result<HealthCheckResponse, tonic::Status>>
            + Send
            + 'static;
        async fn watch(
            &self,
            req: tonic::Request<HealthCheckRequest>,
        ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct HealthServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> HealthServer<T> {
        pub fn new(inner: T) -> Self {
            Self {
                inner: Arc::new(inner),
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T> Clone for HealthServer<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T> tonic::server::NamedService for HealthServer<T> {
        const NAME: &'static str = "grpc.health.v1.Health";
    }
    impl<T, B> Service<http::Request<B>> for HealthServer<T>
    where
        T: Health,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/grpc.health.v1.Health/Check" => {
                    struct Wrapper<T: Health>(Arc<T>);
                    impl<T: Health> tonic::server::UnaryService<HealthCheckRequest> for Wrapper<T> {
                        type Response = HealthCheckResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            req: tonic::Request<HealthCheckRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as Health>::check(&inner, req).await })
                        }
                    }
                    let method = Wrapper(self.inner.clone());
                    let codec = prust::tonic_codec::Codec::default();
                    let mut grpc = tonic::server::Grpc::new(codec)
                        .apply_compression_config(
                            self.accept_compression_encodings,
                            self.send_compression_encodings,
                        )
                        .apply_max_message_size_config(
                            self.max_decoding_message_size,
                            self.max_encoding_message_size,
                        );

                    Box::pin(async move { Ok(grpc.unary(method, req).await) })
                }
                "/grpc.health.v1.Health/Watch" => {
                    struct Wrapper<T: Health>(Arc<T>);
                    impl<T: Health> tonic::server::ServerStreamingService<HealthCheckRequest> for Wrapper<T> {
                        type Response = HealthCheckResponse;
                        type ResponseStream = T::WatchStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            req: tonic::Request<HealthCheckRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            Box::pin(async move { <T as Health>::watch(&inner, req).await })
                        }
                    }
                    let method = Wrapper(self.inner.clone());
                    let codec = prust::tonic_codec::Codec::default();
                    let mut grpc = tonic::server::Grpc::new(codec)
                        .apply_compression_config(
                            self.accept_compression_encodings,
                            self.send_compression_encodings,
                        )
                        .apply_max_message_size_config(
                            self.max_decoding_message_size,
                            self.max_encoding_message_size,
                        );
                    Box::pin(async move { Ok(grpc.server_streaming(method, req).await) })
                }
                _ => Box::pin(async move {
                    let mut resp = http::Response::new(tonic::body::Body::default());
                    let headers = resp.headers_mut();
                    headers.insert(
                        tonic::Status::GRPC_STATUS,
                        (tonic::Code::Unimplemented as i32).into(),
                    );
                    headers.insert(
                        http::header::CONTENT_TYPE,
                        tonic::metadata::GRPC_CONTENT_TYPE,
                    );
                    Ok(resp)
                }),
            }
        }
    }
}
