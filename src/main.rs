#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

mod store;

#[cfg(test)]
mod tests;

use std::sync::Mutex;

use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

use store::{TreeStore, ID};

type SharedTreeStore = Mutex<TreeStore>;

#[derive(Deserialize, Serialize)]
struct Token {
    id: ID,
}

#[derive(Debug, Serialize)]
struct ErrRes {
    error: String,
}

impl From<&str> for ErrRes {
    fn from(error: &str) -> Self {
        ErrRes {
            error: String::from(error),
        }
    }
}

impl<'a> Responder<'a> for ErrRes {
    fn respond_to(self, req: &Request) -> response::Result<'a> {
        Response::build()
            .merge(Json(self).respond_to(req)?)
            .status(Status::BadRequest)
            .ok()
    }
}

#[derive(Deserialize)]
struct SignupReq {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct SignupRes {
    token: Token,
}

#[post("/signup", format = "json", data = "<req>")]
fn signup(req: Json<SignupReq>, store: State<SharedTreeStore>) -> Result<Json<SignupRes>, ErrRes> {
    let mut store = store.lock().expect("can't lock");

    if store.emails.contains_key(&req.email) {
        return Err(ErrRes::from("email exists"));
    }

    // TODO: Check if email exists
    let id = TreeStore::make_id(&req.email);
    //let mut store = store.lock().expect("can't lock");
    //store.0 += 1;
    // TODO: Save to store.emails
    // TODO: Create user
    // TODO: Save to store.users
    Ok(Json(SignupRes {
        token: Token { id },
    }))
}

#[derive(Deserialize)]
struct LoginReq {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginRes {
    token: Token,
}

#[post("/login", format = "json", data = "<req>")]
fn login(req: Json<LoginReq>, store: State<SharedTreeStore>) -> Json<LoginRes> {
    //let mut store = store.lock().expect("can't lock");
    //store.0 += 1;

    // TODO: Don't use make_id here, look it up in store.emails
    // TODO: If not found in store.emails, return an error
    let id = TreeStore::make_id(&req.email);

    // TODO: Verify password

    Json(LoginRes {
        token: Token { id },
    })
}

#[derive(Deserialize)]
struct GetTasksReq {
    token: String,
    tasks: JsonValue,
}

#[get("/", format = "json", data = "<req>")]
fn get_tasks(req: Json<GetTasksReq>, store: State<SharedTreeStore>) {}

#[derive(Deserialize)]
struct PostTasksReq {
    token: Token,
}

#[derive(Serialize)]
struct PostTasksRes {
    tasks: JsonValue,
}

#[post("/", format = "json", data = "<req>")]
fn post_tasks(req: Json<PostTasksReq>, store: State<SharedTreeStore>) -> Json<PostTasksRes> {
    Json(PostTasksRes { tasks: json!(0) })
}

#[catch(400)]
fn bad_request() -> ErrRes {
    ErrRes::from("bad request")
}

#[catch(404)]
fn not_found() -> ErrRes {
    ErrRes::from("not found")
}

#[catch(422)]
fn unprocessable_entity() -> ErrRes {
    ErrRes::from("unprocessable entity")
}

#[catch(500)]
fn internal_server_error() -> ErrRes {
    ErrRes::from("internal server error")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![signup, login])
        .mount("/tasks", routes![get_tasks, post_tasks])
        .register(catchers![
            bad_request,
            not_found,
            unprocessable_entity,
            internal_server_error
        ])
        .manage(Mutex::new(TreeStore::new()))
}

fn main() {
    rocket().launch();
}
