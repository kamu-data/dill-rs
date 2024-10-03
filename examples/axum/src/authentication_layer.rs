// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::response::{IntoResponse, Response};
use axum::RequestExt;
use axum_extra::typed_header::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use tower::{Layer, Service};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub type BearerHeader = TypedHeader<Authorization<Bearer>>;

pub struct Subject {
    pub account_name: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AuthenticationLayer {}

impl AuthenticationLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<Svc> Layer<Svc> for AuthenticationLayer {
    type Service = AuthenticationMiddleware<Svc>;

    fn layer(&self, inner: Svc) -> Self::Service {
        AuthenticationMiddleware { inner }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct AuthenticationMiddleware<Svc> {
    inner: Svc,
}

impl<Svc> Service<http::Request<Body>> for AuthenticationMiddleware<Svc>
where
    Svc: Service<http::Request<Body>, Response = Response> + Send + 'static + Clone,
    Svc::Future: Send + 'static,
{
    type Response = Svc::Response;
    type Error = Svc::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    // Inspired by https://github.com/maxcountryman/axum-login/blob/5239b38b2698a3db3f92075b6ad430aea79c215a/axum-login/src/auth.rs
    fn call(&mut self, mut request: http::Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // This looks for the "Authorization: bearer XXX" header and extracts the bearer
            // token as the subject account name. In a real application we could have a DB
            // call to resolve the access token to an account info.
            let Some(subject) = request
                .extract_parts::<Option<BearerHeader>>()
                .await
                .unwrap()
                .map(|th| Subject {
                    account_name: th.token().to_string(),
                })
            else {
                return Ok((
                    http::StatusCode::UNAUTHORIZED,
                    "Pass 'Authorization: bearer XXX' header",
                )
                    .into_response());
            };

            let base_catalog = request
                .extensions()
                .get::<dill::Catalog>()
                .expect("Catalog not found in http server extensions");

            eprintln!("Authenticated request from {}", subject.account_name);

            // NOTICE: This is the important part!
            //
            // We extract the base catalog out of the axum extensions and create a new
            // chained catalog instance that will contain everything from the base catalog
            // plus the new Subject value. We replace the catalog extension with this new
            // chained instance, effectively creating a request-scoped catalog.
            let request_catalog = dill::CatalogBuilder::new_chained(base_catalog)
                .add_value(subject)
                .build();

            request.extensions_mut().insert(request_catalog);

            inner.call(request).await
        })
    }
}
