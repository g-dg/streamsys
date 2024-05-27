use std::{sync::Arc, time::Duration};

use crate::app::AppState;

/// Runs various maintenance tasks
pub async fn maintenance_tasks(app_state: Arc<AppState>) {
    tokio::join!(
        async {
            tokio::spawn(database_optimize_task(app_state.clone()))
                .await
                .unwrap()
        },
        async {
            tokio::spawn(database_quick_checkpoint_task(app_state.clone()))
                .await
                .unwrap()
        },
        async {
            tokio::spawn(database_full_checkpoint_task(app_state.clone()))
                .await
                .unwrap()
        },
    );
}

/// Runs quick database optimization
pub async fn database_optimize_task(app_state: Arc<AppState>) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(app_state.config.database_maintenance_interval)) => {},
            _ = app_state.shutdown_token.cancelled() => break,
        }

        app_state.database.optimize(false);
    }
}

/// Runs quick database checkpoints
pub async fn database_quick_checkpoint_task(app_state: Arc<AppState>) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(app_state.config.database_quick_checkpoint_interval)) => {},
            _ = app_state.shutdown_token.cancelled() => break,
        }

        app_state.database.checkpoint(false);
    }
}

/// Runs full database checkpoints
pub async fn database_full_checkpoint_task(app_state: Arc<AppState>) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(app_state.config.database_full_checkpoint_interval)) => {},
            _ = app_state.shutdown_token.cancelled() => break,
        }

        app_state.database.checkpoint(true);
    }
}
