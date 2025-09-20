use super::context::{Container, Context};
use crate::ast::{Function, Method, Service};
use crate::codegen::Buffer;
use crate::codegen::sanitize::{snake, upper_camel};

pub fn generate_service(buf: &mut Buffer, svc: &Service, cx: &mut Context) {
    if !cx.config.build_server && !cx.config.build_client {
        println!(
            "cargo::warning=service {:?} founded, but `build_client` and `build_server` are both disabled",
            svc.name
        );
        return;
    }

    if cx.config.build_client {
        generate_client(svc, buf, cx);
    }

    if cx.config.build_server {
        generate_server(svc, buf, cx);
    }
}

fn generate_client(svc: &Service, buf: &mut Buffer, cx: &mut Context) {
    buf.push(format!("pub mod {}_client {{\n", snake(&svc.name)));
    buf.indent += 1;

    buf.push("#![allow(dead_code, unused_imports, unused_variables)]\n");

    buf.push("use super::*;\n");
    buf.push("use tonic::codegen::*;\n");

    // generate
    buf.push("#[derive(Clone, Debug)]\n");
    buf.push(format!(
        "pub struct {}Client<T> {{\n",
        upper_camel(&svc.name)
    ));
    buf.push("    inner: tonic::client::Grpc<T>,\n");
    buf.push("}\n");

    // connect
    buf.push(format!(
        "impl {}Client<tonic::transport::Channel> {{\n",
        upper_camel(&svc.name)
    ));
    buf.indent += 1;
    buf.push("pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>\n");
    buf.push("where\n");
    buf.push("    D: TryInto<tonic::transport::Endpoint>,\n");
    buf.push("    D::Error: Into<StdError>,\n");
    buf.push("{\n");
    buf.push("    let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;\n");
    buf.push("    Ok(Self::new(conn))\n");
    buf.push("}\n");

    buf.indent -= 1;
    buf.push("}\n");

    buf.push(format!("impl<T> {}Client<T>\n", upper_camel(&svc.name)));
    buf.push("where\n");
    buf.push("    T: tonic::client::GrpcService<tonic::body::Body>,\n");
    buf.push("    T::Error: Into<StdError>,\n");
    buf.push("    T::ResponseBody: Body<Data = Bytes> + Send + 'static,\n");
    buf.push("    <T::ResponseBody as Body>::Error: Into<StdError> + Send,\n");
    buf.push("{\n");

    buf.indent += 1;
    buf.push("pub fn new(inner: T) -> Self {\n");
    buf.push("    let inner = tonic::client::Grpc::new(inner);\n");
    buf.push("    Self { inner }\n");
    buf.push("}\n");

    buf.push("pub fn with_origin(inner: T, origin: http::Uri) -> Self {\n");
    buf.push("    let inner = tonic::client::Grpc::with_origin(inner, origin);\n");
    buf.push("    Self { inner }\n");
    buf.push("}\n");

    buf.push("pub fn with_interceptor<F>(\n");
    buf.push("    inner: T,\n");
    buf.push("    interceptor: F,\n");
    buf.push(format!(
        ") -> {}Client<InterceptedService<T, F>>\n",
        upper_camel(&svc.name)
    ));
    buf.push("where\n");
    buf.push("    F: tonic::service::Interceptor,\n");
    buf.push("    T::ResponseBody: Default,\n");
    buf.push("    T: Service<\n");
    buf.push("        http::Request<tonic::body::Body>,\n");
    buf.push("        Response = http::Response<\n");
    buf.push("            <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,\n");
    buf.push("        >,\n");
    buf.push("    >,\n");
    buf.push("    <T as Service<\n");
    buf.push("        http::Request<tonic::body::Body>,\n");
    buf.push("    >>::Error: Into<StdError> + Send + Sync,\n");
    buf.push("{\n");
    buf.push(format!(
        "    {}Client::new(InterceptedService::new(inner, interceptor))\n",
        upper_camel(&svc.name)
    ));
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {\n");
    buf.push("    self.inner = self.inner.send_compressed(encoding);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {\n");
    buf.push("    self.inner = self.inner.accept_compressed(encoding);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn max_decoding_message_size(mut self, limit: usize) -> Self {\n");
    buf.push("    self.inner = self.inner.max_decoding_message_size(limit);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn max_encoding_message_size(mut self, limit: usize) -> Self {\n");
    buf.push("    self.inner = self.inner.max_encoding_message_size(limit);\n");
    buf.push("    self\n");
    buf.push("}\n");

    for func in &svc.functions {
        generate_client_method(svc, func, buf, cx);
    }

    buf.indent -= 1;
    buf.push("}\n");

    buf.indent -= 1;
    buf.push("}\n");
}

fn generate_client_method(svc: &Service, func: &Function, buf: &mut Buffer, cx: &mut Context) {
    let package = cx.fd.package.clone().unwrap_or_default();
    let Some((req, Container::Message(_req))) = cx.lookup_type(&func.request) else {
        panic!("Request type {} not found", func.request);
    };
    let Some((resp, Container::Message(_resp))) = cx.lookup_type(&func.response) else {
        panic!("Response type {} not found", func.request);
    };

    match &func.method {
        Method::Unary => {
            buf.push(format!("pub async fn {}(\n", snake(&func.name)));
            buf.push("    &mut self,\n");

            buf.push(format!("    req: impl tonic::IntoRequest<{}>,\n", req));
            buf.push(") -> Result<\n");
            buf.push(format!("    tonic::Response<{}>,\n", resp));
            buf.push("    tonic::Status\n");
            buf.push("> {\n");
            buf.indent += 1;

            buf.push("self.inner\n");
            buf.push("    .ready()\n");
            buf.push("    .await\n");
            buf.push("    .map_err(|err| tonic::Status::unknown(\n");
            buf.push(format!(
                "        format!(\"Service was not ready: {{}}\", err.into())\n"
            ));
            buf.push("    ))?;\n");

            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push(format!(
                "let path = http::uri::PathAndQuery::from_static(\"/{}.{}/{}\");\n",
                package, svc.name, func.name
            ));
            buf.push("let mut req = req.into_request();\n");
            buf.push("req.extensions_mut()\n");
            buf.push(format!(
                "    .insert(GrpcMethod::new(\"{}.{}\", \"{}\"));\n",
                package, svc.name, func.name
            ));
            buf.push("self.inner.unary(req, path, codec).await\n");

            buf.indent -= 1;
            buf.push("}\n");
        }
        Method::ClientStreaming => {
            buf.push(format!("pub async fn {}(\n", snake(&func.name)));
            buf.push("    &mut self,\n");
            buf.push(format!(
                "    req: impl tonic::IntoStreamingRequest<Message = {}>,\n",
                req
            ));
            buf.push(format!(
                ") -> Result<tonic::Response<{}>, tonic::Status> {{\n",
                resp
            ));
            buf.indent += 1;

            buf.push("self.inner\n");
            buf.push("    .ready()\n");
            buf.push("    .await\n");
            buf.push("    .map_err(|err| tonic::Status::unknown(\n");
            buf.push("        format!(\"Service was not ready: {}\", err.into())\n");
            buf.push("    ))?;\n");

            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let path = http::uri::PathAndQuery::from_static(\n");
            buf.push(format!(
                "    \"/{}.{}/{}\",\n",
                package, svc.name, func.name
            ));
            buf.push(");\n");
            buf.push("let mut req = req.into_streaming_request();\n");
            buf.push("req.extensions_mut()\n");
            buf.push(format!(
                "    .insert(GrpcMethod::new(\"{}.{}\", \"{}\"));\n",
                package, svc.name, func.name
            ));
            buf.push("self.inner.client_streaming(req, path, codec).await\n");

            buf.indent -= 1;
            buf.push("}\n");
        }
        Method::ServerStreaming => {
            buf.push(format!("pub async fn {}(\n", snake(&func.name)));
            buf.push("    &mut self,\n");
            buf.push(format!("    req: impl tonic::IntoRequest<{}>,\n", req));
            buf.push(") -> Result<\n");
            buf.push(format!(
                "    tonic::Response<tonic::codec::Streaming<{}>>,\n",
                resp
            ));
            buf.push("    tonic::Status,\n");
            buf.push("> {\n");
            buf.indent += 1;

            buf.push("self.inner\n");
            buf.push("    .ready()\n");
            buf.push("    .await\n");
            buf.push("    .map_err(|err| tonic::Status::unknown(\n");
            buf.push("        format!(\"Service was not ready: {}\", err.into())\n");
            buf.push("    ))?;\n");

            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let path = http::uri::PathAndQuery::from_static(\n");
            buf.push(format!(
                "    \"/{}.{}/{}\",\n",
                package, svc.name, func.name
            ));
            buf.push(");\n");

            buf.push("let mut req = req.into_request();\n");
            buf.push("req.extensions_mut()\n");
            buf.push(format!(
                "    .insert(GrpcMethod::new(\"{}.{}\", \"{}\"));\n",
                package, svc.name, func.name
            ));
            buf.push("self.inner.server_streaming(req, path, codec).await\n");

            buf.indent -= 1;
            buf.push("}\n");
        }
        Method::BidiStreaming => {
            buf.push(format!("pub async fn {}(\n", snake(&func.name)));
            buf.push("    &mut self,\n");
            buf.push(format!(
                "    req: impl tonic::IntoStreamingRequest<Message = {}>,\n",
                req
            ));
            buf.push(") -> Result<\n");
            buf.push(format!(
                "    tonic::Response<tonic::codec::Streaming<{}>>,\n",
                resp
            ));
            buf.push("    tonic::Status,\n");
            buf.push("> {\n");
            buf.indent += 1;

            buf.push("self.inner\n");
            buf.push("    .ready()\n");
            buf.push("    .await\n");
            buf.push("    .map_err(|err| tonic::Status::unknown(\n");
            buf.push("        format!(\"Service was not ready: {}\", err.into())\n");
            buf.push("    ))?;\n");

            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let path = http::uri::PathAndQuery::from_static(\n");
            buf.push(format!(
                "    \"/{}.{}/{}\",\n",
                package, svc.name, func.name
            ));
            buf.push(");\n");

            buf.push("let mut req = req.into_streaming_request();\n");
            buf.push("req.extensions_mut()\n");
            buf.push(format!(
                "    .insert(GrpcMethod::new(\"{}.{}\", \"{}\"));\n",
                package, svc.name, func.name
            ));
            buf.push("self.inner.streaming(req, path, codec).await\n");

            buf.indent -= 1;
            buf.push("}\n");
        }
    }
}

fn generate_server(svc: &Service, buf: &mut Buffer, cx: &mut Context) {
    let package = cx.fd.package.clone().unwrap_or_default();

    buf.push(format!("pub mod {}_server {{\n", snake(&svc.name)));
    buf.indent += 1;

    buf.push("#![allow(dead_code, unused_imports)]\n");

    buf.push("use super::*;\n");
    buf.push("use tonic::codegen::*;\n");

    buf.push("#[async_trait]\n");
    buf.push(format!(
        "pub trait {}: Send + Sync + 'static {{\n",
        upper_camel(&svc.name)
    ));
    buf.indent += 1;

    // generate service trait
    for func in &svc.functions {
        let Some((req, Container::Message(_req))) = cx.lookup_type(&func.request) else {
            panic!("Request type {} not found", func.request);
        };
        let Some((resp, Container::Message(_resp))) = cx.lookup_type(&func.response) else {
            panic!("Response type {} not found", func.request);
        };

        match &func.method {
            Method::Unary => {
                buf.push(format!("async fn {}(\n", snake(&func.name)));
                buf.push("    &self,\n");
                buf.push(format!("    req: tonic::Request<{}>,\n", req));
                buf.push(format!(
                    ") -> Result<tonic::Response<{}>, tonic::Status>;\n",
                    resp
                ));
            }

            Method::ClientStreaming => {
                buf.push(format!("async fn {}(\n", snake(&func.name)));
                buf.push("    &self,\n");
                buf.push(format!(
                    "    req: tonic::Request<tonic::Streaming<{}>>,\n",
                    req
                ));
                buf.push(format!(
                    ") -> Result<tonic::Response<{}>, tonic::Status>;\n",
                    resp
                ));
            }
            Method::ServerStreaming => {
                buf.push(format!(
                    "type {}Stream: tokio_stream::Stream<\n",
                    upper_camel(&func.name)
                ));
                buf.push(format!("    Item = Result<{}, tonic::Status>,\n", resp));
                buf.push("> + Send + 'static;\n");

                buf.push(format!("async fn {}(\n", snake(&func.name)));
                buf.push("    &self,\n");
                buf.push(format!("    req: tonic::Request<{}>,\n", req));
                buf.push(format!(
                    ") -> Result<tonic::Response<Self::{}Stream>, tonic::Status>;\n",
                    upper_camel(&func.name)
                ));
            }
            Method::BidiStreaming => {
                buf.push(format!(
                    "type {}Stream: tokio_stream::Stream<\n",
                    upper_camel(&func.name)
                ));
                buf.push(format!("    Item = Result<{}, tonic::Status>,\n", resp));
                buf.push("> + Send + 'static;\n");

                buf.push(format!("async fn {}(\n", snake(&func.name)));
                buf.push("    &self,\n");
                buf.push(format!(
                    "    req: tonic::Request<tonic::Streaming<{}>>,\n",
                    req
                ));
                buf.push(format!(
                    ") -> Result<tonic::Response<Self::{}Stream>, tonic::Status>;\n",
                    upper_camel(&func.name)
                ));
            }
        }
    }
    buf.indent -= 1;
    buf.push("}\n");

    // service server struct
    buf.push("#[derive(Debug)]\n");
    buf.push(format!(
        "pub struct {}Server<T> {{\n",
        upper_camel(&svc.name)
    ));
    buf.push("    inner: Arc<T>,\n");
    buf.push("    accept_compression_encodings: EnabledCompressionEncodings,\n");
    buf.push("    send_compression_encodings: EnabledCompressionEncodings,\n");
    buf.push("    max_decoding_message_size: Option<usize>,\n");
    buf.push("    max_encoding_message_size: Option<usize>,\n");
    buf.push("}\n");

    buf.push(format!("impl<T> {}Server<T> {{\n", upper_camel(&svc.name)));
    buf.indent += 1;

    buf.push("pub fn new(inner: T) -> Self {\n");
    buf.push("    Self {\n");
    buf.push("        inner: Arc::new(inner),\n");
    buf.push("        accept_compression_encodings: Default::default(),\n");
    buf.push("        send_compression_encodings: Default::default(),\n");
    buf.push("        max_decoding_message_size: None,\n");
    buf.push("        max_encoding_message_size: None,\n");
    buf.push("    }\n");
    buf.push("}\n");

    buf.push("pub fn with_interceptor<F>(\n");
    buf.push("    inner: T,\n");
    buf.push("    interceptor: F,\n");
    buf.push(") -> InterceptedService<Self, F>\n");
    buf.push("where\n");
    buf.push("    F: tonic::service::Interceptor,\n");
    buf.push("{\n");
    buf.push("    InterceptedService::new(Self::new(inner), interceptor)\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {\n");
    buf.push("    self.accept_compression_encodings.enable(encoding);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {\n");
    buf.push("    self.send_compression_encodings.enable(encoding);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn max_decoding_message_size(mut self, limit: usize) -> Self {\n");
    buf.push("    self.max_decoding_message_size = Some(limit);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.push("#[must_use]\n");
    buf.push("pub fn max_encoding_message_size(mut self, limit: usize) -> Self {\n");
    buf.push("    self.max_encoding_message_size = Some(limit);\n");
    buf.push("    self\n");
    buf.push("}\n");

    buf.indent -= 1;
    buf.push("}\n");

    // ---- Clone ----
    buf.push(format!(
        "impl<T> Clone for {}Server<T> {{\n",
        upper_camel(&svc.name)
    ));
    buf.push("    fn clone(&self) -> Self {\n");
    buf.push("        Self {\n");
    buf.push("            inner: self.inner.clone(),\n");
    buf.push("            accept_compression_encodings: self.accept_compression_encodings,\n");
    buf.push("            send_compression_encodings: self.send_compression_encodings,\n");
    buf.push("            max_decoding_message_size: self.max_decoding_message_size,\n");
    buf.push("            max_encoding_message_size: self.max_encoding_message_size,\n");
    buf.push("        }\n");
    buf.push("    }\n");
    buf.push("}\n");

    buf.push(format!(
        "impl<T> tonic::server::NamedService for {}Server<T> {{\n",
        upper_camel(&svc.name)
    ));
    buf.push(format!(
        "    const NAME: &'static str = \"{}.{}\";\n",
        package,
        upper_camel(&svc.name)
    ));
    buf.push("}\n");

    buf.push(format!(
        "impl<T, B> Service<http::Request<B>> for {}Server<T>\n",
        upper_camel(&svc.name)
    ));
    buf.push("where\n");
    buf.push(format!("    T: {},\n", upper_camel(&svc.name)));
    buf.push("    B: Body + Send + 'static,\n");
    buf.push("    B::Error: Into<StdError> + Send + 'static,\n");
    buf.push("{\n");

    {
        // service implementation
        buf.indent += 1;

        buf.push("type Response = http::Response<tonic::body::Body>;\n");
        buf.push("type Error = std::convert::Infallible;\n");
        buf.push("type Future = BoxFuture<Self::Response, Self::Error>;\n");

        buf.push(
            "fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {\n",
        );
        buf.push("    Poll::Ready(Ok(()))\n");
        buf.push("}\n");

        buf.push("fn call(&mut self, req: http::Request<B>) -> Self::Future {\n");
        buf.indent += 1;
        buf.push("match req.uri().path() {\n");
        for func in &svc.functions {
            // match content
            buf.indent += 1;
            generate_server_handle(svc, func, buf, cx);
            buf.indent -= 1;
        }

        // 404 Not Found
        {
            buf.indent += 1;

            buf.push("_ => Box::pin(async move {\n");
            buf.indent += 1;
            buf.push("let mut resp = http::Response::new(\n");
            buf.push("    tonic::body::Body::default(),\n");
            buf.push(");\n");

            buf.push("let headers = resp.headers_mut();\n");

            buf.push("headers.insert(\n");
            buf.push("    tonic::Status::GRPC_STATUS,\n");
            buf.push("    (tonic::Code::Unimplemented as i32).into(),\n");
            buf.push(");\n");

            buf.push("headers.insert(\n");
            buf.push("    http::header::CONTENT_TYPE,\n");
            buf.push("    tonic::metadata::GRPC_CONTENT_TYPE,\n");
            buf.push(");\n");

            buf.push("Ok(resp)\n");
            buf.indent -= 1;
            buf.push("})\n");

            buf.indent -= 1;
        }

        buf.push("}\n");
        buf.indent -= 1;
        buf.push("}\n");

        buf.indent -= 1;
    }

    buf.push("}\n");

    buf.indent -= 1;
    buf.push("}\n");
}

fn generate_server_handle(svc: &Service, func: &Function, buf: &mut Buffer, cx: &mut Context) {
    let package = cx.fd.package.clone().unwrap_or_default();
    let Some((req, Container::Message(_req))) = cx.lookup_type(&func.request) else {
        panic!("Request type {} not found", func.request);
    };
    let Some((resp, Container::Message(_resp))) = cx.lookup_type(&func.response) else {
        panic!("Response type {} not found", func.request);
    };

    buf.push(format!(
        "\"/{}.{}/{}\" => {{\n",
        package,
        upper_camel(&svc.name),
        upper_camel(&func.name)
    ));
    buf.indent += 1;

    match &func.method {
        Method::Unary => {
            buf.push(format!(
                "struct Wrapper<T: {}>(Arc<T>);\n",
                upper_camel(&svc.name)
            ));
            buf.push(format!(
                "impl<T: {}> tonic::server::UnaryService<{}> for Wrapper<T> {{\n",
                upper_camel(&svc.name),
                req
            ));
            buf.push(format!("    type Response = {};\n", resp));
            buf.push(
                "    type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;\n",
            );

            buf.indent += 1;
            buf.push(format!(
                "fn call(&mut self, req: tonic::Request<{}>) -> Self::Future {{\n",
                req
            ));
            buf.push("    let inner = Arc::clone(&self.0);\n");
            buf.push("    Box::pin(async move {\n");
            buf.push(format!(
                "        <T as {}>::{}(&inner, req).await\n",
                upper_camel(&svc.name),
                snake(&func.name)
            ));
            buf.push("    })\n");
            buf.push("}\n");
            buf.indent -= 1;

            buf.push("}\n");

            buf.push("let method = Wrapper(self.inner.clone());\n");
            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let mut grpc = tonic::server::Grpc::new(codec)\n");
            buf.push("    .apply_compression_config(\n");
            buf.push("        self.accept_compression_encodings,\n");
            buf.push("        self.send_compression_encodings,\n");
            buf.push("    )\n");
            buf.push("    .apply_max_message_size_config(\n");
            buf.push("        self.max_decoding_message_size,\n");
            buf.push("        self.max_encoding_message_size,\n");
            buf.push("    );\n");
            buf.push("\n");

            buf.push("Box::pin(async move {\n");
            buf.push("    Ok(grpc.unary(method, req).await)\n");
            buf.push("})\n");
        }
        Method::ClientStreaming => {
            buf.push(format!(
                "struct Wrapper<T: {}>(Arc<T>);\n",
                upper_camel(&svc.name)
            ));
            buf.push(format!(
                "impl<T: {}> tonic::server::ClientStreamingService<{}> for Wrapper<T> {{\n",
                upper_camel(&svc.name),
                req
            ));
            buf.indent += 1;

            buf.push(format!("type Response = {};\n", resp));
            buf.push("type Future = BoxFuture<\n");
            buf.push("    tonic::Response<Self::Response>,\n");
            buf.push("    tonic::Status,\n");
            buf.push(">;\n");

            buf.push("fn call(\n");
            buf.push("    &mut self,\n");
            buf.push(format!(
                "    req: tonic::Request<tonic::Streaming<{}>>,\n",
                req
            ));
            buf.push(") -> Self::Future {\n");
            buf.push("    let inner = Arc::clone(&self.0);\n");
            buf.push("    Box::pin(async move {\n");
            buf.push(format!(
                "        <T as {}>::{}(&inner, req).await\n",
                upper_camel(&svc.name),
                snake(&func.name)
            ));
            buf.push("    })\n");
            buf.push("}\n");

            buf.indent -= 1;
            buf.push("}\n");

            buf.push("let method = Wrapper(self.inner.clone());\n");
            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let mut grpc = tonic::server::Grpc::new(codec)\n");
            buf.push("    .apply_compression_config(\n");
            buf.push("        self.accept_compression_encodings,\n");
            buf.push("        self.send_compression_encodings,\n");
            buf.push("    )\n");
            buf.push("    .apply_max_message_size_config(\n");
            buf.push("        self.max_decoding_message_size,\n");
            buf.push("        self.max_encoding_message_size,\n");
            buf.push("    );\n");
            buf.push("\n");

            buf.push("Box::pin(async move {\n");
            buf.push("    Ok(grpc.client_streaming(method, req).await)\n");
            buf.push("})\n")
        }
        Method::ServerStreaming => {
            buf.push(format!(
                "struct Wrapper<T: {}>(Arc<T>);\n",
                upper_camel(&svc.name)
            ));
            buf.push(format!(
                "impl<T: {}> tonic::server::ServerStreamingService<{}> for Wrapper<T> {{\n",
                upper_camel(&svc.name),
                req
            ));
            buf.indent += 1;

            buf.push(format!("type Response = {};\n", resp));
            buf.push(format!(
                "type ResponseStream = T::{}Stream;\n",
                upper_camel(&func.name)
            ));
            buf.push("type Future = BoxFuture<\n");
            buf.push("    tonic::Response<Self::ResponseStream>,\n");
            buf.push("    tonic::Status,\n");
            buf.push(">;\n");

            buf.push("fn call(\n");
            buf.push("    &mut self,\n");
            buf.push(format!("    req: tonic::Request<{}>,\n", req));
            buf.push(") -> Self::Future {\n");
            buf.push("    let inner = Arc::clone(&self.0);\n");
            buf.push("    Box::pin(async move {\n");
            buf.push(format!(
                "        <T as {}>::{}(&inner, req).await\n",
                upper_camel(&svc.name),
                snake(&func.name)
            ));
            buf.push("    })\n");
            buf.push("}\n");

            buf.indent -= 1;
            buf.push("}\n");

            buf.push("let method = Wrapper(self.inner.clone());\n");
            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let mut grpc = tonic::server::Grpc::new(codec)\n");
            buf.push("    .apply_compression_config(\n");

            buf.push("    self.accept_compression_encodings,\n");
            buf.push("    self.send_compression_encodings,\n");
            buf.push(")\n");
            buf.push(".apply_max_message_size_config(\n");
            buf.push("    self.max_decoding_message_size,\n");
            buf.push("    self.max_encoding_message_size,\n");
            buf.push(");\n");

            buf.push("Box::pin(async move {\n");
            buf.push("    Ok(grpc.server_streaming(method, req).await)\n");
            buf.push("})\n");
        }
        Method::BidiStreaming => {
            buf.push(format!(
                "struct Wrapper<T: {}>(Arc<T>);\n",
                upper_camel(&svc.name)
            ));
            buf.push(format!(
                "impl<T: {}> tonic::server::StreamingService<{}> for Wrapper<T> {{\n",
                upper_camel(&svc.name),
                req
            ));
            buf.indent += 1;

            buf.push(format!("type Response = {};\n", resp));
            buf.push(format!(
                "type ResponseStream = T::{}Stream;\n",
                upper_camel(&func.name)
            ));
            buf.push("type Future = BoxFuture<\n");
            buf.push("    tonic::Response<Self::ResponseStream>,\n");
            buf.push("    tonic::Status,\n");
            buf.push(">;\n");

            buf.push("fn call(\n");
            buf.push("    &mut self,\n");
            buf.push(format!(
                "    req: tonic::Request<tonic::Streaming<{}>>,\n",
                req
            ));
            buf.push(") -> Self::Future {\n");
            buf.push("    let inner = Arc::clone(&self.0);\n");
            buf.push("    Box::pin(async move {\n");
            buf.push(format!(
                "        <T as {}>::{}(&inner, req).await\n",
                upper_camel(&svc.name),
                snake(&func.name)
            ));
            buf.push("    })\n");
            buf.push("}\n");

            buf.indent -= 1;
            buf.push("}\n");

            buf.push("let method = Wrapper(self.inner.clone());\n");
            buf.push("let codec = prust::tonic_codec::Codec::default();\n");
            buf.push("let mut grpc = tonic::server::Grpc::new(codec)\n");
            buf.push("    .apply_compression_config(\n");
            buf.push("        self.accept_compression_encodings,\n");
            buf.push("        self.send_compression_encodings,\n");
            buf.push("    )\n");
            buf.push("    .apply_max_message_size_config(\n");
            buf.push("         self.max_decoding_message_size,\n");
            buf.push("         self.max_encoding_message_size,\n");
            buf.push("     );\n");

            buf.push("Box::pin(async move {\n");
            buf.push("    Ok(grpc.streaming(method, req).await)\n");
            buf.push("})\n");
        }
    }

    buf.indent -= 1;
    buf.push("}\n");
}
