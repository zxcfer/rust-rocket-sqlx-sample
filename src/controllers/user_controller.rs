use std::io::IntoInnerError;

use crate::app::AppState;
use crate::db::ConnectionDb;
use crate::dto::user_dto::UserName;
use crate::error::app_error::AppError;
use crate::models::user_model::User;
use rocket::serde::json::Json;
use tracing::instrument;

#[get("/")]
#[instrument(name = "user_controller/index", skip_all)]
async fn index(app: &AppState, mut db: ConnectionDb) -> Result<Json<Vec<User>>, AppError> {
    let users = app.use_cases.user.find_all(&app.repos, &mut db).await?;
    Ok(Json(users))
}

// post request with json body
#[post("/add", data = "<user_json>")]
#[instrument(name = "user_controller/add", skip_all)]
async fn add(
    app: &AppState,
    mut db: ConnectionDb,
    user_json: Json<UserName>,
) -> Result<Json<User>, AppError> {

    tracing::info!("==Stating processing...");
    let inner = user_json.into_inner();
    
    tracing::info!("Received request to add user. Name: {}, Age: {}", inner.name, inner.age);

    let name = inner.name;
    let age = inner.age;

    // send 400 if age > 32
    if age > 32 {
        return Err(AppError::new(400, "age must be less than 32"));
    }

    let user = app
        .use_cases
        .user
        .create(&app.repos, &mut db, &name, age)
        .await?;
    Ok(Json(user))
}

#[put("/<id>", data = "<user_json>")]
#[instrument(name = "user_controller/update", skip_all, fields(id = %id))]
async fn update(
    app: &AppState,
    mut db: ConnectionDb,
    id: i32,
    user_json: Json<UserName>,
) -> Result<Json<User>, AppError> {

    let inner = user_json.into_inner();
    let name = inner.name;
    let age = inner.age;


    let user = app
        .use_cases
        .user
        .update(&app.repos, &mut db, id, &name, age)
        .await?;
    Ok(Json(user))
}

#[delete("/<id>")]
#[instrument(name = "user_controller/delete", skip_all, fields(id = %id))]
async fn delete(app: &AppState, mut db: ConnectionDb, id: i32) -> Result<(), AppError> {
    app.use_cases.user.delete(&app.repos, &mut db, id).await?;
    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, add, update, delete]
}

#[cfg(test)]
mod tests {
    use crate::app_err;
    use crate::config::Config;
    use crate::db::Db;
    use crate::test::app::create_app_for_test;
    use crate::test::fixture::user::users_fixture;
    use crate::use_cases::user_use_case::MockUserUseCase;
    use rocket::fairing::AdHoc;
    use rocket::http::Status;
    use rocket::local::asynchronous::Client;
    use rocket_db_pools::Database;
    use std::sync::Arc;

    #[rocket::async_test]
    async fn test_index_success() {
        let mut mock_user_use_case = MockUserUseCase::new();
        mock_user_use_case
            .expect_find_all()
            .returning(|_, _| Ok(users_fixture(5)));

        let mut app_state = create_app_for_test();
        app_state.use_cases.user = Box::new(mock_user_use_case);

        let rocket = rocket::build()
            .manage(Arc::new(app_state))
            .attach(Db::init())
            .attach(AdHoc::config::<Config>())
            .mount("/", routes![super::index]);
        let client = Client::tracked(rocket)
            .await
            .expect("valid rocket instance");
        let response = client.get("/").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn test_index_fail() {
        let mut mock_user_use_case = MockUserUseCase::new();
        mock_user_use_case
            .expect_find_all()
            .returning(|_, _| app_err!(500, "error!"));

        let mut app_state = create_app_for_test();
        app_state.use_cases.user = Box::new(mock_user_use_case);

        let rocket = rocket::build()
            .manage(Arc::new(app_state))
            .attach(Db::init())
            .attach(AdHoc::config::<Config>())
            .mount("/", routes![super::index]);
        let client = Client::tracked(rocket)
            .await
            .expect("valid rocket instance");
        let response = client.get("/").dispatch().await;

        assert_eq!(response.status(), Status::InternalServerError);
        let body_str = response.into_string().await.expect("valid body string");
        assert_eq!(body_str, "error!");
    }
}
