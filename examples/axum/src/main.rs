mod authentication_layer;
use std::sync::Arc;

use authentication_layer::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// API Layer
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn root_handler(
    axum::Extension(catalog): axum::Extension<dill::Catalog>,
) -> impl axum::response::IntoResponse {
    // TODO: We should add a custom extractor or a macro to:
    //   a) avoid pulling objects out manually from the catalog
    //   b) allow us to somehow analyze the handler dependencies to make them a part
    //      of catalog verification
    let greeter = catalog.get_one::<dyn Greeter>().unwrap();

    greeter.greet()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Domain Layer
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

trait Greeter: Send + Sync {
    fn greet(&self) -> String;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Infra Layer
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[dill::component]
#[dill::interface(dyn Greeter)]
struct GreeterImpl {
    subject: Arc<Subject>,
}

impl Greeter for GreeterImpl {
    fn greet(&self) -> String {
        format!(
            "GreeterImpl::greet -> \"Hello, {}\"",
            self.subject.account_name
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() {
    let catalog = dill::Catalog::builder().add::<GreeterImpl>().build();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5456")
        .await
        .unwrap();

    let app = axum::Router::new()
        .route("/", axum::routing::get(root_handler))
        .layer(AuthenticationLayer::new())
        .layer(axum::Extension(catalog));

    eprintln!("Listening on http://{}", listener.local_addr().unwrap());

    eprintln!(
        "Try making a request like:\n  xh -v GET 'http://{}' 'Authorization:bearer Bob'",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await
        .unwrap();
}
