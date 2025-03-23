use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    post, put,
    web::Data,
    web::Json,
    web::Path,
    HttpResponse,
};
use derive_more::Display;
use serd::{Deserialize, Serialize};

// Handler function. Tied to path and HTTP method.
// Handler function has to return one of two things. A struct that implements the Responder trait,
// or a result for which the success value implements the Responder trait and the error value
// implements ResponseError
#[get("/task/{task_global_id}")]
pub async fn get_task() -> Json<String> {
    return Json("hello world".to_string());
}
