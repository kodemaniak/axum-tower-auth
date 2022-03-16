use axum::{
    body::{Body, BoxBody},
    http::{Request, Response, StatusCode},
    routing::get,
    Router,
};
use futures::future::BoxFuture;
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route_layer(AsyncRequireAuthorizationLayer::new(MyAuth {}));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone, Copy)]
struct MyAuth;

impl<B> AsyncAuthorizeRequest<B> for MyAuth
where
    B: Send + Sync + 'static,
{
    type RequestBody = B;
    type ResponseBody = BoxBody;
    type Future = BoxFuture<'static, Result<Request<B>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<B>) -> Self::Future {
        Box::pin(async {
            if let Some(user_id) = check_auth(&request).await {
                // Set `user_id` as a request extension so it can be accessed by other
                // services down the stack.
                request.extensions_mut().insert(user_id);

                Ok(request)
            } else {
                let unauthorized_response = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(axum::body::boxed(Body::empty()))
                    .unwrap();

                Err(unauthorized_response)
            }
        })
    }
}

async fn check_auth<B>(_request: &Request<B>) -> Option<UserId> {
    None
}

#[derive(Debug)]
struct UserId(String);
