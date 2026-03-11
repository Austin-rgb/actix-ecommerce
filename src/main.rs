use crate::configs::{EventMessanger, OrdersInventoryAgent, run_migrations};
use actix_web::{App, HttpServer};
use auth::{AuthModule, SetupError};
use catalog::{CatalogModule, Config as CatalogConfig};
use dotenvy::dotenv;
use inventory::{CreateItemOnInventory, InventoryModule};
use messaging::MessagingModule;
use orders::{Config, OrdersModule};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::{env, process::exit};
use tenant::{AuthorizModule, InitError};
mod configs;

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
    let module = match AuthModule::new(db.clone()).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error occured in setting up auth module. diagnosing...");
            match e {
                SetupError::Db(e) => eprintln!("Error in database: {}", e),
                SetupError::Var(e) => eprintln!("Error in getting env var: {}", e),
            };
            exit(1)
        }
    };
    let authoriz = match AuthorizModule::new(db.clone()).await {
        Ok(r) => r,
        Err(e) => {
            match e {
                InitError::DbConnection(error) => {
                    eprintln!("error in connecting to database: {}", error)
                }
                InitError::DbInit(error) => eprintln!("Error in initializing database: {}", error),
                InitError::Secret(var_error) => {
                    eprintln!("Error in getting SECRET env var: {}", var_error)
                }
                InitError::Permissions(_read_error) => {
                    eprintln!("Error in reading permissions vars",)
                }
            }
            eprintln!("Failed to initialize permissions module");
            exit(1)
        }
    };
    let messages = match MessagingModule::new(db.clone()).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("failed to initialize messaging module: {}", e);
            panic!()
        }
    };

    let inventory = match InventoryModule::new(db.clone()).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error in initializing inventory module: {}", e);
            panic!()
        }
    };
    let catalog_perms = CatalogModule::get_permissions();
    let catalog_perms = authoriz
        .add_permissions("*".to_string(), catalog_perms)
        .await
        .expect("an error occured in adding catalog's perms");
    let catalog_config = CatalogConfig::new()
        .with_on_create(Box::new(CreateItemOnInventory {
            service: inventory.service.clone(),
        }))
        .with_perms(catalog_perms);
    let catalog = match CatalogModule::new(catalog_config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to initialize catalog module: {}", e);
            panic!()
        }
    };

    let orders_config = Config::new()
        .with_event_handler(Box::new(EventMessanger {
            messenger: messages.clone(),
        }))
        .with_inventory_agent(Box::new(OrdersInventoryAgent {
            inventory_module: inventory.clone(),
        }));
    let orders = match OrdersModule::new(orders_config).await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error occured in initializing orders module: {}", e);
            panic!()
        }
    };

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    println!("Starting server on http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| module.config(cfg, "auth"))
            .configure(|cfg| authoriz.config(cfg, "permissions"))
            .configure(|cfg| messages.config(cfg, "messages"))
            .configure(|cfg| catalog.config(cfg, "catalog"))
            .configure(|cfg| orders.config(cfg, "orders"))
            .configure(|cfg| inventory.config(cfg, "inventory"))
    })
    .bind(&bind_address)?
    .run()
    .await
}
