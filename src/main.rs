#![feature(let_chains)]

pub mod common;
pub mod data;
pub mod view;
pub mod state;
pub mod error;
pub mod session;
pub mod tr;
use axum::Router;
use clap::{Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::{signal, task::AbortHandle};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::EnvFilter;

use crate::{data::Repository, state::{default_load_config, init_app_state}};
/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "app")]
#[command(about = "App", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// run application
    #[command()]
    Start {},
    /// run application
    #[command()]
    Migrate {},
    /// create user
    #[command()]
    CreateUser {
        /// username to add
        #[arg(required = true)]
        user: String,
        /// password to add
        #[arg(required = true)]
        password: String,
    },
    UpdatePassword {
        /// username to add
        #[arg(required = true)]
        user: String,
        /// password to add
        #[arg(required = true)]
        password: String,
    },
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = default_load_config().await?;
    let state = Arc::new(init_app_state(&config).await?);

    match args.command {
        Commands::Start {} => {
            // 1. Initialize tracing + log bridging
            tracing_subscriber::fmt()
                // This allows you to use, e.g., `RUST_LOG=info` or `RUST_LOG=debug`
                // when running the app to set log levels.
                .with_env_filter(
                    EnvFilter::try_from_default_env()
                        .or_else(|_| EnvFilter::try_new(config.log.as_str()))
                        .or_else(|_| EnvFilter::try_new("tis100-pad=error,tower_http=warn"))
                        .unwrap(),
                )
                .init();

            let session_store = PostgresStore::new(
                PgPoolOptions::new()
                    .max_connections(5)
                    .connect(config.db.url()?)
                    .await?,
            );
            session_store.migrate().await?;
            let deletion_task = tokio::task::spawn(
                session_store
                    .clone()
                    .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
            );

            let session_layer = SessionManagerLayer::new(session_store)
                .with_always_save(true)
                .with_expiry(Expiry::OnInactivity(
                    tower_sessions::cookie::time::Duration::days(7),
                ));
            // build our application with a route
            let app = Router::new()
                //.route("/", get(view::chart::patient_list_root))
                .nest_service("/static", ServeDir::new("static"))
                .layer(session_layer)
                .layer(TraceLayer::new_for_http())
                .with_state(state);

            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
                .await?;
            deletion_task.await??;
            Ok(())
        }
        Commands::Migrate {} => {
            let pool = PgPoolOptions::new().connect(config.db.url()?).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok(())
        }
        Commands::CreateUser { user, password } => {
            let pool = PgPoolOptions::new().connect(config.db.url()?).await?;
            let mut tx = pool.begin().await?;
            let repo = Repository::from(&pool);
            if let Err(e) = repo
                .create_user(
                    &mut tx,
                    state.password_salt.as_str(),
                    user.as_str(),
                    password.as_str(),
                )
                .await
            {
                return Err(Box::from(e));
            }
            tx.commit().await?;
            Ok(())
        }
        Commands::UpdatePassword { user, password } => {
            let pool = PgPoolOptions::new().connect(config.db.url()?).await?;
            let mut tx = pool.begin().await?;
            let repo = Repository::from(&pool);
            let res = match repo.get_user_by_username(&mut tx, user.as_str()).await {
                Err(e) => {
                    println!("failed to fetch user by {} - {}", user, e);
                    return Err(Box::from(e));
                }
                Ok(res) => res,
            };
            if let Err(e) = repo
                .update_password(
                    &mut tx,
                    res.user_id.0,
                    config.password_salt.as_str(),
                    password.as_str(),
                )
                .await
            {
                return Err(Box::from(e));
            }
            tx.commit().await?;
            Ok(())
        }
    }
}

