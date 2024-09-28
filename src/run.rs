use crate::parser::Parser;
use crate::scorpion::Scorpion; 
use crate::Application;
use anyhow::Result;
use tokio::task::JoinHandle;

/// Run the application by starting scorpions and parsers based on the configuration in `Application`.

/// # Arguments
/// * `app` - A reference to the `Application` instance containing configuration and state.
pub fn run(app: &Application) -> Result<()> {
    let mut tasks: Vec<JoinHandle<Result<()>>> = Vec::new();

    // Start the scorpions
    for _ in 0..app.args.connections {
        let mut scorpion = new_scorpion(app);
        tokio::spawn(async move {
            scorpion.run().await;
        });
    }

     // Start the parsers
    for _ in 0..app.args.parsers {
        let mut parser = new_parser(app);
        tokio::spawn(async move {
            parser.run().await;
        });
    }

    Ok(())

    // Await all tasks to ensure they complete before exiting the application
    // for task in tasks {
    //     if let Err(e) = task.await {
    //         eprintln!("Task failed: {:?}", e);
    //     }
    // }

    // Ok(())
}

fn new_scorpion(app: &Application) -> Scorpion {
    Scorpion::new( 
        app.controller.subscribe(),
        app.rate_limiter.clone(),
        app.send_response.clone(),
        app.receive_request.clone(),
    )
}

fn new_parser(app: &Application) -> Parser {
    Parser::new(
        app.controller.subscribe(),
        app.send_request.clone(),
        app.receive_response.clone(),
        app.index.clone(),
    )
}
