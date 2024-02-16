pub mod database;
pub mod error;
pub mod rdb;
pub mod request;
pub mod resp2;
pub mod response;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerMode {
    Master,
    Slave,
}
