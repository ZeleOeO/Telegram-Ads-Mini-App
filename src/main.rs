use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, Set};
use sea_orm::{Database, DatabaseConnection, EntityTrait, QueryFilter};
use std::env;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use tracing::info;
use std::sync::Arc;
use crate::{entity::users, models::errors::BotResult, services::grammers_client::GrammersClient};
pub mod auth;
pub mod entity;
pub mod handlers;
pub mod models;
pub mod services;
pub mod helpers;
use migration::{Migrator, MigratorTrait};
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Start the bot and create an account.")]
    Start(String),
}
#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
    bot: Bot,
    grammers: Arc<GrammersClient>,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    if database_url.is_empty() {
        panic!("DATABASE_URL is set but empty");
    }
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    info!("Database connection established");
    Migrator::up(&db, None).await?;
    info!("Migrations applied successfully");
    let bot = Bot::from_env();
    let grammers = GrammersClient::default();
    let grammers_arc = Arc::new(grammers);
    let g_clone = grammers_arc.clone();
    tokio::spawn(async move {
        if let Err(e) = g_clone.connect().await {
            tracing::warn!("Failed to connect Grammers client: {:?}", e);
        }
    });
    let app_state = AppState {
        db: db.clone(),
        bot: bot.clone(),
        grammers: grammers_arc,
    };
    services::scheduler::start_scheduler(app_state.clone()).await;
    let command_handler = handlers::Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_commands);
    let chat_member_handler =
        handlers::Update::filter_my_chat_member().endpoint(handlers::bot::handle_my_chat_member);
    tokio::spawn(async move {
        Dispatcher::builder(
            bot,
            dptree::entry()
                .branch(command_handler)
                .branch(chat_member_handler),
        )
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    });
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/me", get(handlers::me_handler))
        .route("/users/wallet", post(handlers::users::update_wallet_address))
        .route("/channels", post(handlers::channels::add_channel_handler))
        .route("/channels", get(handlers::channels::list_channels))
        .route("/channels/my", get(handlers::channels::get_my_channels))
        .route(
            "/channels/bot-admin",
            get(handlers::channels::get_bot_admin_suggestions),
        )
        .route(
            "/channels/:id",
            get(handlers::channels::update_channel_handler) 
                .put(handlers::channels::update_channel_handler)
                .delete(handlers::channels::delete_channel),
        )
        .route(
            "/channels/:id/refresh-stats",
            post(handlers::channels::refresh_channel_stats),
        )
        .route(
            "/channels/:id/ad-formats",
            post(handlers::channels::add_ad_format).get(handlers::channels::get_channel_ad_formats),
        )
        .route(
            "/channels/:channel_id/ad-formats/:format_id",
            delete(handlers::channels::delete_ad_format),
        )
        .route(
            "/channels/:id/pr-managers",
            post(handlers::channels::add_pr_manager),
        )
        .route("/campaigns", post(handlers::campaigns::create_campaign))
        .route("/campaigns", get(handlers::campaigns::list_campaigns))
        .route("/campaigns/my", get(handlers::campaigns::get_my_campaigns))
        .route(
            "/campaigns/:id",
            get(handlers::campaigns::get_campaign)
                .put(handlers::campaigns::edit_campaign)
                .delete(handlers::campaigns::delete_campaign),
        )
        .route(
            "/campaigns/:campaign_id/channels/:channel_id/apply",
            post(handlers::campaigns::apply_to_campaign),
        )
        .route(
            "/campaigns/:id/applications",
            get(handlers::campaigns::get_campaign_applications),
        )
        .route(
            "/campaigns/applications/:id/status",
            post(handlers::campaigns::update_application_status),
        )
        .route("/deals", post(handlers::deals::create_deal))
        .route("/deals/my", get(handlers::deals::get_my_deals))
        .route("/deals/:id", get(handlers::deals::get_deal))
        .route(
            "/deals/:id/negotiate",
            post(handlers::deals::send_negotiation),
        )
        .route("/deals/:id/accept", post(handlers::deals::accept_deal))
        .route("/deals/:id/reject", post(handlers::deals::reject_deal))
        .route("/deals/:id/draft", post(handlers::deals::submit_draft))
        .route("/deals/:id/review-draft", post(handlers::deals::review_draft))
        .route(
            "/deals/:id/creative",
            post(handlers::deals::submit_creative),
        )
        .route(
            "/deals/:id/creative/review",
            post(handlers::deals::review_creative),
        )
        .route(
            "/deals/:id/negotiations",
            get(handlers::deals::get_negotiations),
        )
        .route("/deals/:id/post", post(handlers::deals::trigger_auto_post))
        .route("/deals/:id/mark-paid", post(handlers::deals::mark_paid))
        .route(
            "/deals/:id/confirm-payment",
            post(handlers::deals::confirm_payment),
        )
        .route("/deals/:id/mark-posted", post(handlers::deals::mark_posted))
        .route("/deals/:id/verify-post", post(handlers::deals::verify_post))
        .route(
            "/deals/:id/payment",
            post(handlers::payments::initiate_payment),
        )
        .route(
            "/deals/:id/payment/status",
            get(handlers::payments::get_payment_status),
        )
        .route(
            "/deals/:id/payment/verify",
            post(handlers::payments::verify_payment),
        )
        .route(
            "/deals/:id/transactions",
            get(handlers::payments::get_transactions),
        )
        .route(
            "/deals/:id/payment/release",
            post(handlers::payments::release_funds),
        )
        .fallback_service(
            ServeDir::new("frontend/dist")
                .not_found_service(ServeFile::new("frontend/dist/index.html")),
        )
        .layer(middleware::from_fn(skip_ngrok_browser_warning))
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Web server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
async fn handle_commands(
    bot: Bot,
    db: DatabaseConnection,
    msg: Message,
    cmd: Command,
) -> BotResult<()> {
    match cmd {
        Command::Start(payload) => {
            if let Some(user) = msg.from() {
                let telegram_id = user.id.0 as i64;
                let existing_user = users::Entity::find()
                    .filter(users::Column::TelegramId.eq(telegram_id))
                    .one(&db)
                    .await?;
                if existing_user.is_none() {
                    let new_user = users::ActiveModel {
                        telegram_id: Set(telegram_id),
                        ..Default::default()
                    };
                    new_user.insert(&db).await?;
                    info!("New user created with Telegram ID: {}", telegram_id);
                }
                if !payload.is_empty() {
                    if payload.starts_with("deal_") {
                        let deal_id = payload.replace("deal_", "");
                        bot.send_message(
                            msg.chat.id,
                            format!("Let's negotiate Deal #{}! Type your message or offer here, and I will forward it to the other party.", deal_id),
                        )
                        .await?;
                    } else {
                        bot.send_message(msg.chat.id, format!("Welcome! You started: {}", payload))
                            .await?;
                    }
                } else {
                    bot.send_message(
                        msg.chat.id,
                        "Welcome! Your account is active. Use the menu to open the marketplace.",
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}
async fn skip_ngrok_browser_warning(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::HeaderName::from_static("ngrok-skip-browser-warning"),
        HeaderValue::from_static("true"),
    );
    response
}
