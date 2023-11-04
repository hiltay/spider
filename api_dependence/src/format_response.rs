use axum::{
    http::header,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
pub enum PYQError {
    QueryDataBaseError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    pub fn new(message: &str) -> ErrorResponse {
        ErrorResponse {
            message: message.to_owned(),
        }
    }
}

impl IntoResponse for PYQError {
    fn into_response(self) -> Response {
        let body = match self {
            PYQError::QueryDataBaseError(e) => {
                serde_json::to_string(&ErrorResponse::new(&format!(
                    "查询`posts`表失败,请检查: 1、请求参数是否正确? 2、数据库是否可以连接? 3、posts表是否有数据? 4、字段格式是否正确?  Error: {e}"
                )))
                .unwrap()
            }
        };
        ([(header::CONTENT_TYPE, "application/json")], body).into_response()
    }
}
