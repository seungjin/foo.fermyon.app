use spin_sdk::{
    http::{IntoResponse, Params, Request, Response, Router},
    http_component,
};

#[http_component]
async fn handle_route(req: Request) -> Response {
    println!("------");
    for (key, value) in std::env::vars() {
        println!("{key}: {value}");
    }

    let mut router = Router::new();
    router.get("/bar", bar);
    router.handle(req)
}

fn bar(_req: Request, _param: Params) -> anyhow::Result<impl IntoResponse> {
    Ok(Response::new(200, format!("foo dot bar")))
}
