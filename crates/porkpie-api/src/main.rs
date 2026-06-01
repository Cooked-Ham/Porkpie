use porkpie_api::{build_router, config::Config, db, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    db::upsert_api_key(&pool, &config.api_key).await?;

    let app = build_router(AppState { pool });
    let listener = tokio::net::TcpListener::bind(config.listen_addr()).await?;
    println!("Server listening on {}", config.listen_addr());
    axum::serve(listener, app).await?;

    Ok(())
}
