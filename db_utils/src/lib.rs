use std::env;

use anyhow::{Context, anyhow};
use sqlx::{Error as SqlxError, PgPool, postgres::PgPoolOptions};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Clone)]
struct DbConfig {
    db_host: String,
    db_port: String,
    db_user: String,
    db_password: String,
    db_name: String,
}

impl DbConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            db_host: require_env("DB_HOST")?,
            db_port: require_env("DB_PORT")?,
            db_user: require_env("DB_USER")?,
            db_password: require_env("DB_PASSWORD")?,
            db_name: require_env("DB_NAME")?,
        })
    }

    fn app_database_url(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            "postgres", self.db_user, self.db_password, self.db_host, self.db_port, self.db_name
        )
    }

    fn maintenance_database_url(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/postgres",
            "postgres", self.db_user, self.db_password, self.db_host, self.db_port
        )
    }
}

fn require_env(name: &str) -> anyhow::Result<String> {
    env::var(name).with_context(|| format!("missing required environment variable: {name}"))
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

async fn ensure_database_exists(cfg: &DbConfig) -> anyhow::Result<()> {
    let target_db = cfg.db_name.clone();

    if target_db == "postgres" {
        return Ok(());
    }

    let maintenance_url = cfg.maintenance_database_url();

    let maintenance_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&maintenance_url)
        .await
        .with_context(|| format!("failed to connect to maintenance database via {maintenance_url}"))?;

    let sql = format!("CREATE DATABASE {}", quote_identifier(&target_db));
    match sqlx::query(&sql).execute(&maintenance_pool).await {
        Ok(_) => Ok(()),
        Err(SqlxError::Database(db_err)) if db_err.code().as_deref() == Some("42P04") => Ok(()),
        Err(err) => Err(anyhow!(err).context(format!("failed to create database '{target_db}'"))),
    }
}


pub async fn init_database() -> anyhow::Result<PgPool> {
    let cfg = DbConfig::from_env()?;
    ensure_database_exists(&cfg).await?;

    let database_url = cfg.app_database_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Migrations run at startup and only apply pending versions.
    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
