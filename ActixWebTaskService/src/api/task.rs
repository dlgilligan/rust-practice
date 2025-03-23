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
use serde::{Deserialize, Serialize};

// Field name has to match that of the path parameter
#[derive(Serialize, Deserialize)]
pub struct TaskIdentifier {
    task_global_id: String,
}

// Handler function. Tied to path and HTTP method.
// Handler function has to return one of two things. A struct that implements the Responder trait,
// or a result for which the success value implements the Responder trait and the error value
// implements ResponseError.
// Extractors are a means of getting at various arts of the request from within the handler
// function. We use extractors by adding parameters to the handler function and if those parameters
// implement the FromRequest trait, thats considered an extractor and actix web framework will
// automatically populate those parameters with the appropriate values.
#[get("/task/{task_global_id}")]
pub async fn get_task(task_identifier: Path<TaskIdentifier>) -> Json<String> {
    return Json(task_identifier.into_inner().task_global_id);
}
