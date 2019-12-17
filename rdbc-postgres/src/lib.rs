//! Postgres RDBC Driver
//!
//! This crate implements an RDBC Driver for the `postgres` crate.
//!
//! The RDBC (Rust DataBase Connectivity) API is loosely based on the ODBC and JDBC standards.
//!
//! ```rust,ignore
//! use rdbc_postgres::PostgresDriver;
//! let driver = PostgresDriver::new();
//! let conn = driver.connect("postgres://postgres@localhost:5433");
//! let stmt = conn.create_statement("SELECT foo FROM bar").unwrap();
//! let rs = stmt.execute_query().unwrap();
//! let mut rs = rs.borrow_mut();
//! while rs.next() {
//!   println!("{}", rs.get_string(1));
//! }
//! ```

use rdbc;

use std::cell::RefCell;
use std::rc::Rc;

use postgres::rows::Rows;
use postgres::{Connection, TlsMode};
use rdbc::ResultSet;

pub struct PostgresDriver {}

impl PostgresDriver {
    pub fn new() -> Self {
        PostgresDriver {}
    }

    pub fn connect(&self, url: &str) -> Rc<RefCell<dyn rdbc::Connection>> {
        let conn = postgres::Connection::connect(url, TlsMode::None).unwrap();
        Rc::new(RefCell::new(PConnection::new(conn)))
    }
}

struct PConnection {
    conn: Rc<Connection>,
}

impl PConnection {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Rc::new(conn),
        }
    }
}

impl<'a> rdbc::Connection<'a> for PConnection {

    fn execute_query(&mut self, sql: &str) -> rdbc::Result<Rc<RefCell<dyn ResultSet + 'a>>> {
        let rows: Rows = self.conn.query(sql, &[]).unwrap();
        Ok(Rc::new(RefCell::new(PResultSet { i: 0, rows })))
    }

    fn execute_update(&mut self, sql: &str) -> rdbc::Result<usize> {
        unimplemented!()
    }
}

struct PResultSet {
    i: usize,
    rows: Rows,
}

impl rdbc::ResultSet for PResultSet {

    fn next(&mut self) -> bool {
        if self.i + 1 < self.rows.len() {
            self.i = self.i + 1;
            true
        } else {
            false
        }
    }

    fn get_i32(&self, i: usize) -> Option<i32> {
        self.rows.get(self.i).get(i)
    }

    fn get_string(&self, i: usize) -> Option<String> {
        self.rows.get(self.i).get(i)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::borrow::BorrowMut;

    #[test]
    fn it_works() {
        let driver = PostgresDriver::new();
        let conn = driver.connect("postgres://rdbc:secret@127.0.0.1:5433");
        let mut conn = conn.as_ref().borrow_mut();
        let rs = conn.execute_query("SELECT 1").unwrap();
        let mut rs = rs.as_ref().borrow_mut();
        while rs.next() {
            println!("{:?}", rs.get_i32(1))
        }
    }
}
