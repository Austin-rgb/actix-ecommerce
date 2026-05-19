use actix_web::{App, HttpServer};
use auth::Module as AuthModule;
use cart::Module as CartModule;
use catalog::CatalogModule;
use dotenvy::dotenv;
use ferrumec::di::Inject;
use ferrumec::di::EnvContext;
use inventory::InventoryModule;
use notification::Module as NotificationModule;
use orders::Module as OrdersModule;
use std::{env, process::exit};
use tenant::AuthorizModule;

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod logging;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let di_ctx = EnvContext::default();
    let module = match di_ctx.inject(AuthModule::new).await {
        Ok(m) => m.await,
        Err(e) => {
            tracing::error!("Error occured in setting up auth module: {}", e);

            exit(1)
        }
    };
    let authoriz = match di_ctx.inject(AuthorizModule::new).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to initialize permissions module: {}", e);
            exit(1)
        }
    };

    let inventory = match di_ctx.inject(InventoryModule::new).await {
        Ok(r) => match r.await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("failed to initialize inventory module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            tracing::error!("Env di failed for inventory module: {}", e);
            panic!()
        }
    };

    let catalog = match di_ctx.inject(CatalogModule::new).await {
        Ok(c) => match c.await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("failed to initialize catalog module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            tracing::error!("failed to initialize catalog module: {}", e);
            panic!()
        }
    };

    let orders = match di_ctx.inject(OrdersModule::new).await {
        Ok(o) => match o.await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to initialize orders module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            tracing::error!("Env di failed orders module: {}", e);
            panic!()
        }
    };

    let notifiyer = match di_ctx.inject(NotificationModule::new).await {
        Ok(r) => r.await,
        Err(e) => {
            tracing::error!("failed to initialize notifications module: {}", e);
            panic!()
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
