use actix_web::{App, HttpServer};
use auth::Module as AuthModule;
use cart::Module as CartModule;
use catalog::CatalogModule;
use dotenvy::dotenv;
use ferrumec::Dependencies;
use inventory::InventoryModule;
use notification::Module as NotificationModule;
use orders::Module as OrdersModule;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use std::{env, process::exit};
use tenant::AuthorizModule;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
mod logging;
use actixutils::{Authority, HS256Signer, Identity, Sign, Validate};
use emailgrid::{EmailAddress, EmailingContext, Resend, Sender};
use event_stream::{EventStream, NatsEventStream};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool: Pool<Sqlite> = SqlitePool::connect(&url)
        .await
        .expect("Could not connect to db");
    let mut di_ctx = Dependencies::new();
    di_ctx.insert(pool);

    let jwt = Arc::new(HS256Signer::new(
        env::var("validate.aud").expect("validate.aud not set"),
        env::var("validate.secret").expect("validate.secret not set"),
    ));

    di_ctx.insert(jwt.clone() as Arc<dyn Validate<Identity>>);
    di_ctx.insert(jwt.clone() as Arc<dyn Validate<Authority>>);
    di_ctx.insert(jwt.clone() as Arc<dyn Sign<Identity>>);
    di_ctx.insert(jwt as Arc<dyn Sign<Authority>>);
    let name = env::var("email.name").expect("email.name not set");
    let email = env::var("email.address").expect("email.address not set");
    let email = EmailAddress { name, email };
    let sender = Arc::new(Resend::new().expect("could not load resend")) as Arc<dyn Sender>;
    let ec = EmailingContext::new(sender, email).unwrap();
    di_ctx.insert(ec);
    let nats = match NatsEventStream::new(&env::var("es.url").expect("es.url not set")).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Could not connect to event stream: {e}");
            exit(1)
        }
    };
    let es = Arc::new(nats) as Arc<dyn EventStream>;
    di_ctx.insert(es);

    let module = match di_ctx.inject(AuthModule::new) {
        Ok(r) => r.await,
        Err(e) => {
            tracing::error!("Failed to di for auth module: {}", e);
            exit(1)
        }
    };
    let authoriz = match di_ctx.inject(AuthorizModule::new) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to di for permissions module: {}", e);
            exit(1)
        }
    };

    let inventory = match di_ctx.inject(InventoryModule::new) {
        Ok(r) => match r.await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Could not initialize inventory: {e}");
                exit(1)
            }
        },
        Err(e) => {
            tracing::error!("Could not di for inventory: {e}");
            exit(1)
        }
    };

    let catalog = match di_ctx.inject(CatalogModule::new) {
        Ok(r) => match r.await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Could not initialize catalog module: {e}");
                exit(1)
            }
        },
        Err(e) => {
            tracing::error!("Could not di for catalog module; {e}");
            exit(1)
        }
    };

    let orders = match di_ctx.inject(OrdersModule::new) {
        Ok(r) => match r.await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Could not initialize orders module: {e}");
                exit(1)
            }
        },
        Err(e) => {
            tracing::error!("Could not di for orders module; {e}");
            exit(1)
        }
    };

    let notifiyer = match di_ctx.inject(NotificationModule::new) {
        Ok(r) => r.await,
        Err(e) => {
            tracing::error!("Failed to initialize notifications module: {}", e);
            exit(1)
        }
    };

    let cart = CartModule::new();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    println!("Starting server on http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .wrap(logging::LoggingMiddleware)
            .configure(|cfg| module.config(cfg, "auth"))
            .configure(|cfg| authoriz.config(cfg, "permissions"))
            .configure(|cfg| catalog.config(cfg, "catalog"))
            .configure(|cfg| orders.config(cfg, "orders"))
            .configure(|cfg| inventory.config(cfg, "inventory"))
            .configure(|cfg| cart.config(cfg, "cart"))
            .configure(|cfg| notifiyer.config(cfg, "notifications"))
    })
    .bind(&bind_address)?
    .run()
    .await
}
