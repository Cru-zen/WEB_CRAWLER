use crate::index::Index;
use crate::messages::{Request, Response};
use crate::urls::get_urls;

use async_channel::{Receiver, Sender};
use dashmap::mapref::entry::Entry::{Occupied, Vacant};
use dashmap::DashSet;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use tokio::select;
use tokio_utils::ShutdownMonitor;
use tracing::debug;
use url::Url;

/// Parsers are responsible for parsing HTML and finding the next URLs to crawl.
/// They pass URLs to the Scorpions.

/// The Parsers build the index of URLs and their associated links.
pub struct Parser {
    /// The ID of the parser.
    _id: usize,
    /// The channel to receive HTML from.
    receiver: Receiver<Response>,
    /// The channel to send URLs to crawl to.
    sender: Sender<Request>,
    /// The shutdown listener.
    shutdown: ShutdownMonitor,
    /// The index.
    index: Arc<Index>,
}

impl Parser {
    pub fn new(
        shutdown: ShutdownMonitor,
        sender: Sender<Request>,
        receiver: Receiver<Response>,
        index: Arc<Index>,
    ) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self {
            _id: COUNTER.fetch_add(1, SeqCst),
            receiver,
            sender,
            shutdown,
            index,
        }
    }

    pub async fn run(&mut self) {
        let Parser {
            _id: _,
            receiver,
            sender,
            shutdown,
            index,
        } = self;

        // Run the parser until the shutdown signal is received.
        select! {
           _ = shutdown.recv() => { }
           _ = do_work(receiver, sender, index) => { }
        }
    }
}

pub async fn do_work(receiver: &Receiver<Response>, sender: &Sender<Request>, index: &Arc<Index>) {
    loop {
        // Pull the next response off of the channel
        let res = receiver.recv().await;

        // Unwrap the response
        let response = match res {
            Ok(response) => response,
            Err(err) => {
                debug!("Failure: {}", err);
                continue;
            }
        };

        // Get the text of the HTML from the response
        let text = match response.response.text().await {
            Ok(text) => text,
            Err(err) => {
                debug!("Failed to get text from response: {}", err);
                continue;
            }
        };

        // Get the name of the URL that was crawled
        let starting_url = response.source.to_string();

        // Check if we've already processed this URL
        match index.inner.entry(starting_url.clone()) {
            Occupied(_entry) => continue,
            Vacant(entry) => {
                // Add an empty set to put URLs in
                entry.insert(DashSet::new());
            }
        }

        // Get all of the URLs from the HTML
        let urls = match get_urls(response.source, &text) {
            Ok(urls) => urls,
            Err(err) => {
                debug!("Failed to get URLs from HTML: {}", err);
                continue;
            }
        };

        // Parse the URLs for the scorpion requests. The parsing ensures the URLs are valid.
        let parsed_urls: Vec<Url> = urls
            .iter()
            .map(|u| Url::parse(u))
            .filter_map(|u| u.ok())
            .collect::<Vec<_>>();

        // Convert the URLs to strings. These will go into the index.
        let url_strings = parsed_urls
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>();

        // Send URLs to the scorpion only if we haven't seen them before
        for url in parsed_urls {
            if !index.inner.contains_key(url.as_str()) {
                let _ = sender.send(Request { url }).await;
            } else {
                debug!("Skipping {} because it's already in the index", url)
            }
        }

        // Associate the set of URLs with the host URL
        match index.inner.entry(starting_url.clone()) {
            Occupied(entry) => {
                url_strings.iter().for_each(|url| {
                    entry.get().insert(url.to_string());
                });
            }
            Vacant(_entry) => unreachable!(),
        }
    }
}
