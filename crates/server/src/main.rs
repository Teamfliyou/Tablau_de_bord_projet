use tdb_server::{router, AppState, Settings, Store};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let settings = Settings::depuis_env()?;
    let store = Store::ouvrir(&settings.db_path)
        .map_err(|e| anyhow::anyhow!("Ouverture de la base impossible : {e}"))?;
    let bind = settings.bind;
    let frontend = settings.frontend_dir.clone();
    let etat = AppState::nouveau(settings, store)?;

    match &frontend {
        Some(d) => tracing::info!("Frontend servi depuis {}", d.display()),
        None => tracing::warn!("Aucun frontend trouvé (FRONTEND_DIR) : seule l'API est servie."),
    }
    let ecouteur = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("Serveur à l'écoute sur http://{bind}");
    axum::serve(ecouteur, router(etat))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Arrêt demandé.");
        })
        .await?;
    Ok(())
}
