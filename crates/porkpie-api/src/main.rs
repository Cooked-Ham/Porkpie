use porkpie_api::{build_router, config::Config, db, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--healthcheck" {
        let config = Config::from_env()?;
        match tokio::net::TcpStream::connect(config.listen_addr()).await {
            Ok(_) => {
                println!("ok");
                return Ok(());
            }
            Err(_) => {
                std::process::exit(1);
            }
        }
    }

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
