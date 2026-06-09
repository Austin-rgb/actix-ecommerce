use actix_web::{App, HttpServer};
use actixutils::{Authority, HS256Signer, Identity, Sign, Validate};
use auth::Module as AuthModule;
use cart::Module as CartModule;
use catalog::CatalogModule;
use dotenvy::dotenv;
use emailgrid::{EmailAddress, EmailingContext, Resend, Sender};
use event_stream::{EventStream, NatsEventStream};
use ferrumec::Dependencies;
use inventory::InventoryModule;
use notification::Module as NotificationModule;
use orders::Module as OrdersModule;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{env, process::exit, sync::Arc};
use tenant::AuthorizModule;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod logging;

fn fatal(msg: impl std::fmt::Display) -> ! {
    tracing::error!("{msg}");
    exit(1);
}

async fn build_pool() -> Pool<Sqlite> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    SqlitePool::connect(&url)
        .await
        .unwrap_or_else(|e| fatal(format!("Could not connect to database: {e}")))
}

fn register_email(deps: &mut Dependencies) {
    let sender = Arc::new(Resend::new().expect("could not load resend")) as Arc<dyn Sender>;

    let from = EmailAddress {
        name: env::var("email.name").expect("email.name not set"),
        email: env::var("email.address").expect("email.address not set"),
    };

    let ctx = EmailingContext::new(sender, from)
        .unwrap_or_else(|e| fatal(format!("Could not initialize email context: {e}")));

    deps.insert(ctx);
}

async fn register_event_stream(deps: &mut Dependencies) {
    let url = env::var("es.url").expect("es.url not set");

    let stream = NatsEventStream::new(&url)
        .await
        .unwrap_or_else(|e| fatal(format!("Could not connect to event stream: {e}")));

    deps.insert(Arc::new(stream) as Arc<dyn EventStream>);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let pool = build_pool().await;

    let mut deps = Dependencies::new();
    deps.insert(pool);

    let jwt = Arc::new(HS256Signer::new(
        env::var("validate.aud").expect("validate.aud not set"),
        env::var("validate.secret").expect("validate.secret not set"),
    ));
    let val_id = jwt.clone() as Arc<dyn Validate<Identity>>;
    let val_auth = jwt.clone() as Arc<dyn Validate<Authority>>;
    deps.insert(val_id.clone());
    deps.insert(val_auth.clone());
    deps.insert(jwt.clone() as Arc<dyn Sign<Identity>>);
    deps.insert(jwt.clone() as Arc<dyn Sign<Authority>>);
    register_email(&mut deps);
    register_event_stream(&mut deps).await;

    let auth = deps
        .inject(AuthModule::new)
        .unwrap_or_else(|e| fatal(format!("Failed to DI auth module: {e}")))
        .await;

    let permissions = deps
        .inject(AuthorizModule::new)
        .unwrap_or_else(|e| fatal(format!("Failed to DI permissions module: {e}")));

    let inventory = deps
        .inject(InventoryModule::new)
        .unwrap_or_else(|e| fatal(format!("Failed to DI inventory module: {e}")))
        .await
        .unwrap_or_else(|e| fatal(format!("Failed to initialize inventory: {e}")));

    let catalog = deps
        .inject(CatalogModule::new)
        .unwrap_or_else(|e| fatal(format!("Failed to DI catalog module: {e}")))
        .await
        .unwrap_or_else(|e| fatal(format!("Failed to initialize catalog: {e}")));

    let orders = deps
        .inject(OrdersModule::new)
        .unwrap_or_else(|e| fatal(format!("Failed to DI orders module: {e}")))
        .await
        .unwrap_or_else(|e| fatal(format!("Failed to initialize orders: {e}")));

    let notifier = deps
        .inject(NotificationModule::new)
        .unwrap_or_else(|e| fatal(format!("Failed to DI notifications module: {e}")))
        .await;

    let cart = CartModule::new();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let bind_address = format!("{host}:{port}");

    tracing::info!("Starting server on http://{bind_address}");

    HttpServer::new(move || {
        App::new()
            .app_data(val_id.clone())
            .app_data(val_auth.clone())
            .wrap(logging::LoggingMiddleware)
            .configure(|cfg| auth.config(cfg, "auth"))
            .configure(|cfg| permissions.config(cfg, "permissions"))
            .configure(|cfg| catalog.config(cfg, "catalog"))
            .configure(|cfg| orders.config(cfg, "orders"))
            .configure(|cfg| inventory.config(cfg, "inventory"))
            .configure(|cfg| cart.config(cfg, "cart"))
            .configure(|cfg| notifier.config(cfg, "notifications"))
    })
    .bind(&bind_address)?
    .run()
    .await
}
