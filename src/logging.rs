use actix_web::dev::{Service, Transform};
use actix_web::{Error, dev::ServiceRequest, dev::ServiceResponse};
use futures::Future;
use futures::future::{Ready, ok};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

// Define the middleware
pub struct LoggingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct LoggingMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Capture request info
        let path = req.path().to_string();
        let method = req.method().to_string();
        let peer_addr = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let start_time = std::time::Instant::now();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed().as_millis();
            let status = res.status().as_u16();

            // Here is where you log
            println!(
                "[LOG] {} {} from {} => {} ({} ms)",
                method, path, peer_addr, status, duration
            );

            Ok(res)
        })
    }
}
