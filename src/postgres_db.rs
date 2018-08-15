pub init(settings: PostgresSettings) -> impl Future<Item=(), Error=AppError> {
    // initializes a db with all necessary tables if it does not exist 
    unimplemented!()
}

pub struct PostgresDb {
    settings: PostgresSettings
}

impl PostgresDb {
    pub fn new(settings: settings::PostgresSettings) -> PostgresDb {
        PostgresDb {
            settings 
        }
    }

    // you should return Option<User>!
    pub fn find_user_by_email(email: &str) -> Box<Future<Item=User, Error=AppError> + Send> {
        let uri = &self.get_uri();
        let cmdText = format!("SELECT id, email, password FROM {}.person where email = $1", &self.settings.schema_name);
        
        Box::new(match Connection::connect(uri, TlsMode::None) {
            Ok(conn) => match &conn.query(cmdText, &[ &email ]) {
                Ok(rows) => if !rows.is_empty() {
                        let row = rows.get(0);
                        println!("found user");
                        ok(User {
                            id: row.get("id"),
                            email: row.get("email"),
                            password: row.get("password")
                        })
                    } else {
                        println!("no user found");
                        err(AppError::Unauthorized)
                },
                Err(error) => { println!("Pos1{}", error); err(AppError::ApplicationError) }
            }
            Err(error) => { println!("Pos2{}", error); err(AppError::ApplicationError) }
        })
    }

    // TODO: upsert user, insert user 
    fn upsertUser(&self, user: User) -> impl Future<Item=User, Error=AppError> + Send {
        ok(user)
    }
    
    fn get_uri(&self) -> String {
        format!("postgres://{}:{}@{}:{}/{}",
            &self.settings.user,
            &self.settings.password,
            &self.settings.host,
            &self.settings.port,
            &self.settings.db_db_name
        )
    }
}