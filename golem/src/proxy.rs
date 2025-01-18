// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Reverse proxy implementation for Golem's single executable mode.
use anyhow::Context;
use hyper::body::Incoming;
use hyper::{service::service_fn, Request};
use hyper_reverse_proxy::call as reverse_proxy;
use hyper_util::{rt::TokioExecutor, server::conn::auto::Builder};
use regex::Regex;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tower::make::Shared;
use tracing::info;

use crate::AllRunDetails;

pub async fn start_proxy(
    listener_addr: &str,
    listener_port: u16,
    healthcheck_port: u16,
    all_run_details: &AllRunDetails,
    join_set: &mut JoinSet<Result<(), anyhow::Error>>,
) -> anyhow::Result<()> {
    info!("Starting proxy");

    let ipv4_addr: Ipv4Addr = listener_addr.parse().context(format!(
        "Failed at parsing the listener host address {}",
        listener_addr
    ))?;
    let listener_socket_addr = SocketAddr::new(ipv4_addr.into(), listener_port);

    let health_backend = format!("http://{}:{}", listener_addr, healthcheck_port);
    let worker_backend = format!(
        "http://{}:{}",
        listener_addr, all_run_details.worker_service.http_port
    );
    let component_backend = format!(
        "http://{}:{}",
        listener_addr, all_run_details.component_service.http_port
    );

    let re_connect = Regex::new(r"^/v1/components/[^/]+/workers/[^/]+/connect$").unwrap();
    let re_workers = Regex::new(r"^/v1/components/[^/]+/workers").unwrap();
    let re_invoke = Regex::new(r"^/v1/components/[^/]+/invoke(?:-and-await)?$").unwrap();

    let service = Shared::new(service_fn(move |req: Request<Incoming>| {
        let path = req.uri().path().to_string();

        // Routing:
        // 1) equals("/healthcheck"), equals("/metrics") -> health
        // 2) regex("/v1/components/[^/]+/workers/[^/]+/connect$") -> worker
        // 3) prefix("/v1/api") -> worker
        // 4) regex("/v1/components/[^/]+/workers") -> worker
        // 5) regex("/v1/components/[^/]+/invoke") -> worker
        // 6) regex("/v1/components/[^/]+/invoke-and-await") -> worker
        // 7) prefix("/v1/components") -> component
        // 8) prefix("/") -> component

        let target = if path == "/healthcheck" || path == "/metrics" {
            &health_backend
        } else if re_connect.is_match(&path) {
            &worker_backend
        } else if path.starts_with("/v1/api") {
            &worker_backend
        } else if re_workers.is_match(&path) {
            &worker_backend
        } else if re_invoke.is_match(&path) {
            &worker_backend
        } else {
            &component_backend
        };

        reverse_proxy(listener_addr, target, req)
    }));

    let server = Builder::new(TokioExecutor::new()).serve(listener_socket_addr, service)?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    join_set.spawn(async move {
        if let Err(e) = server.await {
            tracing::error!("Proxy server error: {}", e);
        }
        let _ = shutdown_tx.send(());
        Ok(())
    });

    shutdown_rx.await.context("Shutdown signal received")?;
    Ok(())
}
