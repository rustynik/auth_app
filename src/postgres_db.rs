extern crate postgres;

use super::settings::PostgresSettings;
use super::core::traits::{StoreUsers, ManagePasswords, Crypto};
use super::core::models::User;
use super::core::errors::AppError;
use futures::future::{Future, err, ok};
use self::postgres::{Connection, TlsMode};

pub fn init_db(settings: &PostgresSettings) {
        // initializes a db with all necessary tables if it does not exist 
        let uri = helpers::get_uri(settings);
        // initialize the right schema, database and tables if they don't exist 
}

mod helpers {
    use super::super::settings::PostgresSettings;

    pub fn get_uri(settings: &PostgresSettings) -> String {
        format!("postgres://{}:{}@{}:{}/{}",
            settings.user,
            settings.password,
            settings.host,
            settings.port,
            settings.db_name
        )
    }
}

pub struct PostgresDb {
        uri: String,
        db_name: String,
        schema: String,
        crypto_service: Box<Crypto>
    }

impl PostgresDb {
    pub fn new(settings: &PostgresSettings, crypto_service: Box<Crypto>) -> Self {
        Self {
            uri: helpers::get_uri(settings),
            db_name: settings.db_name.clone(),
            schema: settings.schema.clone(),
            crypto_service: crypto_service
        }
    }
}

impl StoreUsers for PostgresDb {
        
    fn find_user_by_id(&self, id: &str) -> Future<Item=Option<User>, Error=AppError> + Send {
            
        let cmdText = format!("SELECT id, name FROM {}.users where id = $1", &self.schema_name);
            
            Box::new(match Connection::connect(&self.uri, TlsMode::None) {
                Ok(conn) => match &conn.query(cmdText, &[ &id ]) {
                    Ok(rows) => if !rows.is_empty() {
                                    let row = rows.get(0);
                                    println!("found user");
                                    ok(Some(User {
                                        id: row.get("id"),
                                        name: row.get("name")
                                    }))
                                } else {
                                    println!("no user found");
                                    ok(None)
                                },
                    Err(error) => { 
                        println!("Error retrieving errors from query {}: {}", cmdText, error); 
                        err(AppError::ApplicationError) 
                    }
                },
                Err(error) => { 
                    println!("Error connecting to uri {}: {}", &self.uri, error); 
                    err(AppError::ApplicationError) 
                }
            })
    }
    
    fn insert_user(&self, user: User) -> Future<Item=User, Error=AppError> + Send {
            
            // implementation relies on user id being primary key in the db
        let cmdText = format!("insert into {}.users id, name, password values($1, $2, $3)", &self.schema_name);

        match Connection::connect(&self.uri, TlsMode::None) {
            Ok(conn) => match &conn.execute(cmdText, &[ &user.id, &user.name, &user.password ]) {
                    Ok(_) => ok(user),
                    Err(error) => { 
                        println!("Error executing query {}: {}", cmdText, error); 
                        err(AppError::ApplicationError) 
                    }
            }
        }
    }
}

impl ManagePasswords for PostgresDb {

    fn set_password(&self, user_id: &str, password: &str) -> Future<Item=(), Error=AppError> + Send {
        let hash = &self.crypto_service.encrypt_password(password);

        let cmdText = format!("insert into {}.passwords (user_id, password_hash) values($1, $2)", &self.schema_name);

        match Connection::connect(&self.uri, TlsMode::None) {
            Ok(conn) => match &conn.execute(cmdText, &[ user_id, &hash ]) {
                Ok(_) => ok(()),
                Err(error) => { 
                    println!("Error executing query {}: {}", cmdText, error); 
                    err(AppError::ApplicationError) 
                }
            }
        }
    }

    fn check_password(&self, user_id: &str, password: &str) -> Future<Item=bool, Error=AppError> + Send {
        let hash = &self.crypto_service.encrypt_password(password);

        let cmdText = format!("select count(*) from {}.passwords where user_id = $1 and password_hash = $2", &self.schema_name);

        match Connection::connect(&self.uri, TlsMode::None) {
            Ok(conn) => match &conn.execute_scalar(cmdText, &[ user_id, &hash ]) {
                Ok(1) => ok(true),
                Ok(_) => ok(false),
                Err(error) => { 
                    println!("Error executing query {}: {}", cmdText, error); 
                    err(AppError::ApplicationError) 
                }
            }
        }
    }
}