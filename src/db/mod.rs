use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Executor, PgPool};
use std::str::FromStr;
use std::time::Duration;

/// Crea el pool de conexiones a PostgreSQL.
///
/// El `after_connect` aplica `SET search_path TO game_world` en cada
/// conexión nueva, en lugar de delegarlo al parámetro `options=...` de la URL.
/// Es más robusto cuando se conecta a través de un pooler (Supabase Supavisor,
/// PgBouncer, etc.) que a veces ignora o rechaza los startup parameters.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let opts = PgConnectOptions::from_str(database_url)?;

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET search_path TO game_world").await?;
                Ok(())
            })
        })
        .connect_with(opts)
        .await
}
