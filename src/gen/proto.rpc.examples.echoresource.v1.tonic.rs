// @generated
/// Generated client implementations.
pub mod echo_resource_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///
    #[derive(Debug, Clone)]
    pub struct EchoResourceServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EchoResourceServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> EchoResourceServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Default + Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> EchoResourceServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            EchoResourceServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        ///
        pub async fn echo_resource(
            &mut self,
            request: impl tonic::IntoRequest<super::EchoResourceRequest>,
        ) -> Result<tonic::Response<super::EchoResourceResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.rpc.examples.echoresource.v1.EchoResourceService/EchoResource",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn echo_resource_multiple(
            &mut self,
            request: impl tonic::IntoRequest<super::EchoResourceMultipleRequest>,
        ) -> Result<
                tonic::Response<
                    tonic::codec::Streaming<super::EchoResourceMultipleResponse>,
                >,
                tonic::Status,
            > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.rpc.examples.echoresource.v1.EchoResourceService/EchoResourceMultiple",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        ///
        pub async fn echo_resource_bi_di(
            &mut self,
            request: impl tonic::IntoStreamingRequest<
                Message = super::EchoResourceBiDiRequest,
            >,
        ) -> Result<
                tonic::Response<
                    tonic::codec::Streaming<super::EchoResourceBiDiResponse>,
                >,
                tonic::Status,
            > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/proto.rpc.examples.echoresource.v1.EchoResourceService/EchoResourceBiDi",
            );
            self.inner.streaming(request.into_streaming_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod echo_resource_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with EchoResourceServiceServer.
    #[async_trait]
    pub trait EchoResourceService: Send + Sync + 'static {
        ///
        async fn echo_resource(
            &self,
            request: tonic::Request<super::EchoResourceRequest>,
        ) -> Result<tonic::Response<super::EchoResourceResponse>, tonic::Status>;
        ///Server streaming response type for the EchoResourceMultiple method.
        type EchoResourceMultipleStream: futures_core::Stream<
                Item = Result<super::EchoResourceMultipleResponse, tonic::Status>,
            >
            + Send
            + 'static;
        ///
        async fn echo_resource_multiple(
            &self,
            request: tonic::Request<super::EchoResourceMultipleRequest>,
        ) -> Result<tonic::Response<Self::EchoResourceMultipleStream>, tonic::Status>;
        ///Server streaming response type for the EchoResourceBiDi method.
        type EchoResourceBiDiStream: futures_core::Stream<
                Item = Result<super::EchoResourceBiDiResponse, tonic::Status>,
            >
            + Send
            + 'static;
        ///
        async fn echo_resource_bi_di(
            &self,
            request: tonic::Request<tonic::Streaming<super::EchoResourceBiDiRequest>>,
        ) -> Result<tonic::Response<Self::EchoResourceBiDiStream>, tonic::Status>;
    }
    ///
    #[derive(Debug)]
    pub struct EchoResourceServiceServer<T: EchoResourceService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: EchoResourceService> EchoResourceServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.accept_compression_encodings.enable_gzip();
            self
        }
        /// Compress responses with `gzip`, if the client supports it.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.send_compression_encodings.enable_gzip();
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EchoResourceServiceServer<T>
    where
        T: EchoResourceService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/proto.rpc.examples.echoresource.v1.EchoResourceService/EchoResource" => {
                    #[allow(non_camel_case_types)]
                    struct EchoResourceSvc<T: EchoResourceService>(pub Arc<T>);
                    impl<
                        T: EchoResourceService,
                    > tonic::server::UnaryService<super::EchoResourceRequest>
                    for EchoResourceSvc<T> {
                        type Response = super::EchoResourceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EchoResourceRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).echo_resource(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EchoResourceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.rpc.examples.echoresource.v1.EchoResourceService/EchoResourceMultiple" => {
                    #[allow(non_camel_case_types)]
                    struct EchoResourceMultipleSvc<T: EchoResourceService>(pub Arc<T>);
                    impl<
                        T: EchoResourceService,
                    > tonic::server::ServerStreamingService<
                        super::EchoResourceMultipleRequest,
                    > for EchoResourceMultipleSvc<T> {
                        type Response = super::EchoResourceMultipleResponse;
                        type ResponseStream = T::EchoResourceMultipleStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EchoResourceMultipleRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).echo_resource_multiple(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EchoResourceMultipleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/proto.rpc.examples.echoresource.v1.EchoResourceService/EchoResourceBiDi" => {
                    #[allow(non_camel_case_types)]
                    struct EchoResourceBiDiSvc<T: EchoResourceService>(pub Arc<T>);
                    impl<
                        T: EchoResourceService,
                    > tonic::server::StreamingService<super::EchoResourceBiDiRequest>
                    for EchoResourceBiDiSvc<T> {
                        type Response = super::EchoResourceBiDiResponse;
                        type ResponseStream = T::EchoResourceBiDiStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<super::EchoResourceBiDiRequest>,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).echo_resource_bi_di(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EchoResourceBiDiSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: EchoResourceService> Clone for EchoResourceServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: EchoResourceService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: EchoResourceService> tonic::transport::NamedService
    for EchoResourceServiceServer<T> {
        const NAME: &'static str = "proto.rpc.examples.echoresource.v1.EchoResourceService";
    }
}
