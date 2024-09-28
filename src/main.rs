mod args;
mod index;
mod messages;
mod parser;
mod run;
mod scorpion;
mod urls;

use crate::args::Args;
use crate::index::Index;
use crate::messages::{Request, Response};
use crate::run::run;
use anyhow::{Context, Result};
use async_channel::{Receiver, Sender};
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::{select, signal::ctrl_c};
use tokio_utils::{MultiRateLimiter, ShutdownController};
use tracing::info;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the application
    let app = Application::new().await?;

    run(&app)?;

    // Wait for a shutdown signal & signal the application to shutdown
    select! {
        _ = ctrl_c() => { return app.shutdown().await; }
    }
}

pub struct Application {
    pub args: Args,
    pub controller: ShutdownController,
    pub rate_limiter: Arc<MultiRateLimiter<String>>,
    pub index: Arc<Index>,
    pub send_request: Sender<Request>,
    pub receive_request: Receiver<Request>,
    pub send_response: Sender<Response>,
    pub receive_response: Receiver<Response>,
}

impl Application {
    pub async fn new() -> Result<Self> {
        let mut args = Application::initialize();
        let controller = ShutdownController::new();
        let rate_limiter: Arc<MultiRateLimiter<String>> =
            Arc::new(MultiRateLimiter::new(Duration::from_millis(args.interval)));
        let index = Arc::new(Index::new(args.file.take()));
        let (send_request, receive_request) = async_channel::unbounded();
        let (send_response, receive_response) = async_channel::unbounded();

        // Seed the initial request so that the spiders can kick off the application
        send_request
            .send(Request::new(Url::parse(&args.target)?))
            .await
            .context("Failed to send initial request")?;

        Ok(Self {
            args,
            controller,
            rate_limiter,
            index,
            send_request,
            receive_request,
            send_response,
            receive_response,
        })
    }

    fn initialize() -> Args {
        let args = Args::parse();
        tracing_subscriber::fmt::init();
        info!("Starting up...");
        args
    }

    async fn shutdown(self) -> Result<()> {
        self.controller.shutdown().await;
        info!("Shutting down...");
        Ok(())
    }
}