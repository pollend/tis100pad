pub mod common;
pub mod data;
pub mod error;
pub mod state;
pub mod tr;

use data::user::User;
use fake::Faker;
use fake::{faker::lorem::raw::*, faker::name::raw::*, faker::time::raw::*, Fake};
use fake::{locales::*, Dummy, Opt, Optional, Rng};
use state::default_load_config;

use clap::{command, Parser, Subcommand};
use data::repository::Repository;
use sqlx::postgres::PgPoolOptions;


// dev utilties
#[derive(Debug, Subcommand)]
enum Commands {
    /// run application
    #[command()]
    Faker {},
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "app")]
#[command(about = "App", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = default_load_config().await?;

    match args.command {
        Commands::Faker {} => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(config.db.url()?)
                .await?;
            let mut tx = pool.begin().await?;
            let repo = Repository::from(&pool);
            let mut users: Vec<User> = vec![];

            let en_eng_faker = Name(EN);
            for n in 1..=100 {
                println!("user: {}", n);
                let name: String = en_eng_faker.fake();
                users.push(
                    repo.create_user(
                        &mut tx,
                        &config.password_salt.as_str(),
                        name.as_str(),
                        "password",
                    )
                    .await?,
                );
            }

            tx.commit().await?;
        }
    }
    Ok(())
}

