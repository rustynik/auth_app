use settings;
use core::{User, StoreUsers, ManagePasswords, Crypto};

pub mod init {
    pub init_db(settings: &PostgresSettings) {
        // initializes a db with all necessary tables if it does not exist 
        let uri = helpers::get_uri(settings);
        // initialize the right schema, database and tables if they don't exist 
    }
}

mod helpers {

    fn get_uri(settings: &PostgresSettings) -> String {
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
    pub fn new(settings: &PostgresSettings) -> Self {
        Self {
            uri: get_uri(settings),
            db_name: settings.db_name.clone(),
            schema: settings.schema.clone(),
            crypto_service: Box<Crypto>
        }
    }
}

impl StoreUsers for PostgresDb {
        
    pub fn find_user_by_id(&self, id: &str) -> Box<Future<Item=Option<User>, Error=AppError> + Send> {
            
        let cmdText = format!("SELECT id, name, password FROM {}.users where id = $1", &self.schema_name);
            
            Box::new(match Connection::connect(&self.uri, TlsMode::None) {
                Ok(conn) => match &conn.query(cmdText, &[ &email ]) {
                    Ok(rows) => if !rows.is_empty() {
                                    let row = rows.get(0);
                                    println!("found user");
                                    ok(Some(User {
                                        id: row.get("id"),
                                        email: row.get("email"),
                                        password: row.get("password")
                                    }))
                                } else {
                                    println!("no user found");
                                    ok(None)
                                }
                    },
                    Err(error) => { 
                        println!("Error retrieving errors from query {}: {}", cmdText, error); 
                        err(AppError::ApplicationError) 
                    }
                }
                Err(error) => { 
                    println!("Error connecting to uri {}: {}", &self.uri, error); 
                    err(AppError::ApplicationError) 
        })
    }
    
    pub fn insert_user(&self, user: User) -> impl Future<Item=User, Error=AppError> + Send {
            
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
}

impl ManagePasswords for PostgresDb {

    pub fn set_password(&self, user_id: &str, password: &str) -> Future<Item=(), Error=AppError> {
        let hash = &self.crypto_service.encrypt_password(password);

        let cmdText = format!("insert into {}.passwords (user_id, password_hash) values($1, $2)", &self.schema_name);

        match Connection::connect(&self.uri, TlsMode::None) {
            Ok(conn) => match &conn.execute(cmdText, &[ user_id, &hash ]) {
                Ok(_) => ok(())),
                Err(error) => { 
                    println!("Error executing query {}: {}", cmdText, error); 
                    err(AppError::ApplicationError) 
                }
            }
        }
    }

    pub fn check_password(&self, userId: &str, password: &str) -> Future<Item=bool, Error=AppError> {
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