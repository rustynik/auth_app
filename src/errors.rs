#[derive(Serialize, Deserialize)]
pub enum AppError {
    ApplicationError,
    RoutingError,
    Unauthorized,
    BadRequest
}

impl AppError {
    pub fn from(statusCode: hyper::StatusCode) -> AppError {
        match statusCode {
            hyper::StatusCode::NOT_FOUND => AppError::RoutingError,
            hyper::StatusCode::FORBIDDEN => AppError::Unauthorized,
            _ => AppError::BadRequest
        }
    }

    pub fn to_status(&self) -> StatusCode {
    (match &self {
        AppError::ApplicationError => StatusCode::from_u16(500),
        AppError::RoutingError => StatusCode::from_u16(404),
        AppError::Unauthorized => StatusCode::from_u16(403),
        AppError::BadRequest => StatusCode::from_u16(400)
    }).unwrap()
}
}

impl Debug for AppError {
    pub fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
} 

impl std::convert::From<AppError> for hyper::Error {
    pub fn from(error: AppError) -> Self {
        panic!("I don't know what to do")
    }
}
 
