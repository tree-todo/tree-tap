#![feature(proc_macro_hygiene, decl_macro)]

extern crate argon2;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;
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

use store::{TreeStore, User, ID};

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
            error: error.to_string(),
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

    let id = TreeStore::make_id(&req.email);
    let user = User::new(&req.password);

    store.emails.insert(req.email.clone(), id);
    store.users.insert(id, user);

    let res = SignupRes {
        token: Token { id },
    };
    Ok(Json(res))
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
fn login(req: Json<LoginReq>, store: State<SharedTreeStore>) -> Result<Json<LoginRes>, ErrRes> {
    let store = store.lock().expect("can't lock");

    if !store.emails.contains_key(&req.email) {
        return Err(ErrRes::from("no such user"));
    }

    let id = store.emails.get(&req.email).unwrap().clone();
    let user = store.users.get(&id).unwrap();

    if !user.verify_pw(&req.password) {
        return Err(ErrRes::from("password doesn't match"));
    }

    let res = LoginRes {
        token: Token { id },
    };
    Ok(Json(res))
}

#[derive(Deserialize)]
struct GetTasksReq {
    token: Token,
}

#[derive(Serialize)]
struct GetTasksRes {
    tasks: serde_json::Value,
}

#[get("/", format = "json", data = "<req>")]
fn get_tasks(
    req: Json<GetTasksReq>,
    store: State<SharedTreeStore>,
) -> Result<Json<GetTasksRes>, ErrRes> {
    let store = store.lock().expect("can't lock");

    let id = req.token.id;

    if !store.users.contains_key(&id) {
        return Err(ErrRes::from("invalid token"));
    }

    if !store.tasks.contains_key(&id) {
        return Err(ErrRes::from("no tasks"));
    }
    let tasks = store.tasks.get(&id).unwrap().clone();

    let res = GetTasksRes { tasks };
    Ok(Json(res))
}

#[derive(Deserialize)]
struct PostTasksReq {
    token: Token,
    tasks: serde_json::Value,
}

#[post("/", format = "json", data = "<req>")]
fn post_tasks(req: Json<PostTasksReq>, store: State<SharedTreeStore>) -> Result<String, ErrRes> {
    let mut store = store.lock().expect("can't lock");

    let id = req.token.id;

    if !store.users.contains_key(&id) {
        return Err(ErrRes::from("invalid token"));
    }

    let tasks = store.tasks.insert(id, req.tasks.clone());

    Ok("".to_string())
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
    let default = rocket_cors::CorsOptions::default();
    let cors = default.to_cors().unwrap();
    rocket::ignite()
        .attach(cors)
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
