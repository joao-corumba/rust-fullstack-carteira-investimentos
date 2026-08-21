use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    models::Asset,
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true);

    Ok((jar.add(cookie), Redirect::to("/")))
}

/// A row of the assets table, with money values pre-formatted to a fixed
/// number of decimal places so the template stays free of formatting logic.
struct AssetRow {
    asset: Asset,
    unit_value_formatted: String,
    quantity_formatted: String,
    total_value_formatted: String,
}

impl From<Asset> for AssetRow {
    fn from(asset: Asset) -> Self {
        let unit_value_formatted = format!("{:.2}", asset.unit_value);
        let quantity_formatted = format!("{:.4}", asset.quantity);
        let total_value_formatted = format!("{:.2}", asset.total_value());

        Self {
            asset,
            unit_value_formatted,
            quantity_formatted,
            total_value_formatted,
        }
    }
}
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    username: String,
    total_value_formatted: String,
    assets: Vec<AssetRow>,
}

async fn index(maybe_user: Option<User>, repository: Repository) -> Result<Response, AppError> {
    let user = match maybe_user {
        Some(user) => user,
        None => return Ok(Redirect::to("/login").into_response()),
    };

    let assets = repository.list_assets().await?;
    let total_value: f64 = assets.iter().map(Asset::total_value).sum();

    let page = DashboardPage {
        username: user.username().clone(),
        total_value_formatted: format!("{total_value:.2}"),
        assets: assets.into_iter().map(AssetRow::from).collect(),
    };

    Ok(Html(page.render()?).into_response())
}
