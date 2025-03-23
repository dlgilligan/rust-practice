use crate::{
    model::task::{Task, TaskState},
    repository::ddb::DDBRepository,
};
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

#[derive(Deserialize)]
pub struct TaskCompletionRequest {
    result_file: String,
}

#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    task_type: String,
    source_file: String,
}

// As noted in the Handler function notes below. Handler function can return a Result for which the
// error value implements ResponseError
#[derive(Debug, Display)]
pub enum TaskError {
    TaskNotFound,
    TaskUpdateFailure,
    TaskCreationFailure,
    BadTaskRequest,
}

impl ResponseError for TaskError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TaskError::TaskNotFound => StatusCode::NOT_FOUND,
            TaskError::TaskUpdateFailure => StatusCode::FAILED_DEPENDENCY,
            TaskError::TaskCreationFailure => StatusCode::FAILED_DEPENDENCY,
            TaskError::BadTaskRequest => StatusCode::BAD_REQUEST,
        }
    }
}

// Handler function. Tied to path and HTTP method.
// Handler function has to return one of two things: A struct that implements the Responder trait,
// or a result for which the success value implements the Responder trait and the error value
// implements ResponseError.
// Extractors are a means of getting at various arts of the request from within the handler
// function. We use extractors by adding parameters to the handler function and if those parameters
// implement the FromRequest trait, thats considered an extractor and actix web framework will
// automatically populate those parameters with the appropriate values.
#[get("/task/{task_global_id}")]
pub async fn get_task(
    task_identifier: Path<TaskIdentifier>,
    ddb_repo: Data<DDBRepository>,
) -> Result<Json<Task>, TaskError> {
    let task = ddb_repo
        .get_task(task_identifier.into_inner().task_global_id)
        .await;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(TaskError::TaskNotFound),
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>,
    task_global_id: String,
    new_state: TaskState,
    result_file: Option<String>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    let mut task = match ddb_repo.get_task(task_global_id).await {
        Some(task) => task,
        None => return Err(TaskError::TaskNotFound),
    };

    if !task.can_transition_to(&new_state) {
        return Err(TaskError::BadTaskRequest);
    }

    task.state = new_state;
    task.result_file = result_file;

    let task_identifier = task.get_global_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier {
            task_global_id: task_identifier,
        })),
        Err(_) => Err(TaskError::TaskUpdateFailure),
    }
}

#[put("/task/{task_global_id}/start")]
pub async fn start_task(
    ddb_repo: Data<DDBRepository>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo,
        task_identifier.into_inner().task_global_id,
        TaskState::InProgress,
        None,
    )
    .await
}

#[put("/task/{task_global_id}/pause")]
pub async fn pause_task(
    ddb_repo: Data<DDBRepository>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo,
        task_identifier.into_inner().task_global_id,
        TaskState::Paused,
        None,
    )
    .await
}

#[put("/task/{task_global_id}/fail")]
pub async fn fail_task(
    ddb_repo: Data<DDBRepository>,
    task_identifier: Path<TaskIdentifier>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo,
        task_identifier.into_inner().task_global_id,
        TaskState::Failed,
        None,
    )
    .await
}

#[put("/task/{task_global_id}/complete")]
pub async fn complete_task(
    ddb_repo: Data<DDBRepository>,
    task_identifier: Path<TaskIdentifier>,
    completion_request: Json<TaskCompletionRequest>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo,
        task_identifier.into_inner().task_global_id,
        TaskState::Completed,
        Some(completion_request.result_file.clone()),
    )
    .await
}
