use sqlx::{
    Pool, Sqlite, migrate,
    migrate::{MigrateError, Migrator},
};

pub static MIGRATOR: Migrator = migrate!();

pub async fn run_migrations(db: &Pool<Sqlite>) -> Result<(), MigrateError> {
    MIGRATOR.run(db).await
}
