use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::{Object, Pool};

pub mod models;
pub mod schema;

pub type DbConn = Object<AsyncPgConnection>;
pub type DbPool = Pool<AsyncPgConnection>;