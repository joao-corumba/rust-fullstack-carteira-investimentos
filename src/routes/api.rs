use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState, auth::admin::Admin, error::AppError, models::Asset, repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/assets",
            get(list_assets).post(create_asset).patch(update_asset),
        )
        .route("/assets/total", get(total_portfolio_value))
}

#[tracing::instrument(skip_all)]
async fn list_assets(repostiory: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repostiory.list_assets().await?;
    Ok(Json(assets))
}

#[derive(Serialize)]
struct TotalValueResponse {
    total_value: f64,
}

/// Returns the sum of `unit_value * quantity` across every asset in the wallet,
/// so the dashboard (or any API consumer) can show the portfolio's total value
/// without recomputing it client-side.
#[tracing::instrument(skip_all)]
async fn total_portfolio_value(
    repostiory: Repository,
) -> Result<Json<TotalValueResponse>, AppError> {
    let total_value = repostiory.total_portfolio_value().await?;
    Ok(Json(TotalValueResponse { total_value }))
}

#[derive(Deserialize)]
struct CreateAssetRequest {
    name: String,
    unit_value: f64,
    #[serde(default)]
    quantity: f64,
}

#[tracing::instrument(skip_all)]
async fn create_asset(
    _: Admin,
    repostiory: Repository,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let new_asset = repostiory
        .create_asset(request.name, request.unit_value, request.quantity)
        .await?;

    Ok(Json(new_asset))
}

#[derive(Deserialize)]
struct UpdateAssetRequest {
    id: i64,
    name: Option<String>,
    unit_value: Option<f64>,
    quantity: Option<f64>,
}

#[tracing::instrument(skip_all)]
async fn update_asset(
    _: Admin,
    repostiory: Repository,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    match repostiory
        .update_asset(
            request.id,
            request.name,
            request.unit_value,
            request.quantity,
        )
        .await?
      {
        Some(updated_asset) => Ok(Json(updated_asset)),
        None => Err(AppError::AssetDoesNotExist),
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_create_asset(db: PgPool) {
        let request = CreateAssetRequest {
            name: "Bitcoin".to_string(),
            unit_value: 10.0,
            quantity: 2.0,
        };
        let Json(new_asset) = create_asset(Admin, db.into(), Json(request))
            .await
            .expect("success");

        assert_eq!(new_asset.id, 1);
        assert_eq!(new_asset.name, "Bitcoin");
        assert_eq!(new_asset.unit_value, 10.0);
        assert_eq!(new_asset.quantity, 2.0);

        insta::assert_json_snapshot!(new_asset);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_list_assets(db: PgPool) {
        let Json(assets) = list_assets(db.into()).await.expect("success");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Bitcoin");

        insta::assert_json_snapshot!(assets);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_update_asset(db: PgPool) {
        let request = UpdateAssetRequest {
            id: 1,
            name: Some("Ethereum".to_string()),
            unit_value: Some(20.0),
            quantity: Some(3.0),
        };

        let Json(updated_asset) = update_asset(Admin, db.into(), Json(request))
            .await
            .expect("success");

        assert_eq!(updated_asset.id, 1);
        assert_eq!(updated_asset.name, "Ethereum");
        assert_eq!(updated_asset.unit_value, 20.0);
        assert_eq!(updated_asset.quantity, 3.0);

        insta::assert_json_snapshot!(updated_asset);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_total_portfolio_value(db: PgPool) {
        // fixture holds 1 Bitcoin priced at 10.0 with quantity 4.0 => total 40.0
        let Json(response) = total_portfolio_value(db.into()).await.expect("success");

        assert_eq!(response.total_value, 40.0);
    }
}
