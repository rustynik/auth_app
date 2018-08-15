#[derive(Serialize, Deserialize)]   
pub struct Session {
    id: String
}

fn create_session(user: User) -> Box<Future<Item=Session, Error=AppError> + Send> {
    let session = Session { id: uuid::Uuid::new_v4().to_string() };
    Box::new(match Connection::connect("postgres://postgres:1@localhost:5432/auth", TlsMode::None) {
        Ok(conn) => match &conn.execute("INSERT into public.session (session_id, user_id) values($1,$2)", &[ &session.id, &user.id ]) {
                Ok(_) => ok(session),
                Err(error) => { println!("Pos1{}", error); err(AppError::ApplicationError) }
            }
            Err(error) => { println!("Pos2{}", error); err(AppError::ApplicationError) }
    })
} 