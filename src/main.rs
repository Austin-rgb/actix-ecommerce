use crate::configs::run_migrations;
use actix_web::{App, HttpServer};
use auth::AuthModule;
use cart::Module as CartModule;
use catalog::CatalogModule;
use dotenvy::dotenv;
use ferrumec::di::run_async;
use inventory::InventoryModule;
use messaging::MessagingModule;

use orders::OrdersModule;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::{env, process::exit};
use tenant::AuthorizModule;
mod configs;
mod logging;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db: Pool<Sqlite> = match SqlitePoolOptions::new()
        .connect("sqlite:database.db/?mode=rwc")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("could not connect to database: {}", e);
            exit(1)
        }
    };
    match run_migrations(&db).await {
        Ok(_) => (),
        Err(e) => eprintln!("Error in running migrations: {}", e),
    };
    let module = match run_async(AuthModule::new).await {
        Ok(m) => m.await,
        Err(e) => {
            eprintln!("Error occured in setting up auth module. diagnosing...");

            exit(1)
        }
    };
    let authoriz = match run_async(AuthorizModule::new).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize permissions module");
            exit(1)
        }
    };
    let messages = match run_async(MessagingModule::new).await {
        Ok(m) => match m {
            Ok(r) => r,
            Err(e) => {
                eprintln!("failed to initialize messaging module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("failed to initialize messaging module: {}", e);
            panic!()
        }
    };

    let inventory = match run_async(InventoryModule::new).await {
        Ok(r) => match r.await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("failed to initialize inventory module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("Env di failed for inventory module: {}", e);
            panic!()
        }
    };
    let catalog_perms = CatalogModule::get_permissions();
    let catalog_perms = authoriz
        .add_permissions("*".to_string(), catalog_perms)
        .await
        .expect("an error occured in adding catalog's perms");

    let catalog = match run_async(CatalogModule::new).await {
        Ok(c) => match c.await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("failed to initialize catalog module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("failed to initialize catalog module: {}", e);
            panic!()
        }
    };

    let orders = match run_async(OrdersModule::new).await {
        Ok(o) => match o.await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to initialize orders module: {}", e);
                panic!()
            }
        },
        Err(e) => {
            eprintln!("Env di failed orders module: {}", e);
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
            .configure(|cfg| messages.config(cfg, "messages"))
            .configure(|cfg| catalog.config(cfg, "catalog"))
            .configure(|cfg| orders.config(cfg, "orders"))
            .configure(|cfg| inventory.config(cfg, "inventory"))
            .configure(|cfg| cart.config(cfg, "cart"))
    })
    .bind(&bind_address)?
    .run()
    .await
}
