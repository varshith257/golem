// Copyright 2024 Golem Cloud
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

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use hyper_util::rt::TokioExecutor;
use hyper_util::server::conn::auto::Builder;
use hyper_util::service::TowerToHyperService;

// use sozu_command_lib::proto::command::WorkerResponse;
// use sozu_command_lib::{
//     channel::Channel,
//     config::ListenerBuilder,
//     proto::command::{
//         request::RequestType, AddBackend, Cluster, LoadBalancingAlgorithms, PathRule,
//         RequestHttpFrontend, RulePosition, SocketAddress, WorkerRequest,
//     },
// };
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use tracing::info;

use crate::AllRunDetails;

// Handlers for each route
async fn healthcheck_handler() -> &'static str {
    "OK"
}

async fn metrics_handler() -> &'static str {
    "Metrics endpoint"
}

async fn connect_handler() -> &'static str {
    "Connect handler"
}

async fn api_handler() -> &'static str {
    "API handler"
}

async fn worker_handler() -> &'static str {
    "Worker handler"
}

async fn invoke_handler() -> &'static str {
    "Invoke handler"
}

async fn invoke_and_await_handler() -> &'static str {
    "Invoke and Await handler"
}

async fn components_handler() -> &'static str {
    "Components handler"
}

async fn root_handler() -> &'static str {
    "Root handler"
}

pub async fn start_proxy(
    listener_port: u16,
    healthcheck_port: u16,
    _all_run_details: &AllRunDetails,
    join_set: &mut JoinSet<Result<(), anyhow::Error>>,
) -> Result<(), anyhow::Error> {
    info!("Starting proxy");

    let app = Router::new()
        .route("/healthcheck", get(healthcheck_handler))
        .route("/metrics", get(metrics_handler))
        .route(
            "/v1/components/:component_id/workers/:worker_id/connect",
            post(connect_handler),
        )
        .route("/v1/api", get(api_handler))
        .route("/v1/components/:component_id/workers", get(worker_handler))
        .route("/v1/components/:component_id/invoke", post(invoke_handler))
        .route(
            "/v1/components/:component_id/invoke-and-await",
            post(invoke_and_await_handler),
        )
        .route("/v1/components", get(components_handler))
        .route("/", get(root_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], listener_port));
    info!("Proxy server listening on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind listener")?;
    let builder = Builder::new(TokioExecutor::new());

    // Spawn the main proxy listener loop
    join_set.spawn({
        let app = app.clone();
        let builder = builder.clone();

        async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let service = TowerToHyperService::new(app.clone());
                        let builder = builder.clone();
                        let wrapped_stream = hyper_util::rt::TokioIo::new(stream);
                        tokio::spawn(async move {
                            if let Err(err) =
                                builder.serve_connection(wrapped_stream, service).await
                            {
                                info!("Connection error with {}: {}", addr, err);
                            }
                        });
                    }
                    Err(err) => {
                        info!("Failed to accept connection: {}", err);
                    }
                }
            }
        }
    });

    // Healthcheck listener
    let health_addr = SocketAddr::from(([127, 0, 0, 1], healthcheck_port));
    info!("Healthcheck server listening on {}", health_addr);

    let health_listener = TcpListener::bind(health_addr)
        .await
        .context("Failed to bind healthcheck listener")?;
    let health_router = Router::new().route("/healthcheck", get(healthcheck_handler));

    join_set.spawn({
        let health_router = health_router.clone();
        let builder = builder.clone();

        async move {
            loop {
                match health_listener.accept().await {
                    Ok((stream, addr)) => {
                        let service = TowerToHyperService::new(health_router.clone());
                        let builder = builder.clone();
                        let wrapped_stream = hyper_util::rt::TokioIo::new(stream);
                        tokio::spawn(async move {
                            if let Err(err) =
                                builder.serve_connection(wrapped_stream, service).await
                            {
                                info!("Healthcheck connection error with {}: {}", addr, err);
                            }
                        });
                    }
                    Err(err) => {
                        info!("Failed to accept healthcheck connection: {}", err);
                    }
                }
            }
        }
    });

    Ok(())
}
