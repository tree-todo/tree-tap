#![feature(decl_macro, never_type, proc_macro_hygiene)]

extern crate argon2;
extern crate base64;
#[macro_use]
extern crate rocket;
extern crate rocket_cors;
#[macro_use]
extern crate serde_derive;

mod store;

#[cfg(test)]
mod tests;

use std::sync::Mutex;

use base64::decode;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response};
use rocket::State;
use rocket_contrib::json::Json;

use store::{ID, TreeStore, User};

type SharedTreeStore = Mutex<TreeStore>;

//
// To get around E0117.
// To provide access to Request within a request handler.
// To avoid having to take up the Stack Overflow answer at
//     https://stackoverflow.com/questions/58030378/using-a-custom-rocket-responder-for-an-error-in-a-requestguard.
//
struct RequestProxy<'a, 'r>(&'a Request<'r>);

impl<'a, 'r> FromRequest<'a, 'r> for RequestProxy<'a, 'r> {
    type Error = !;
    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        Outcome::Success(RequestProxy(&request))
    }
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

impl From<String> for ErrRes {
    fn from(error: String) -> Self {
        ErrRes {
            error,
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

#[derive(Deserialize, Serialize)]
struct Token {
    id: ID,
}

impl<'a, 'r> Token {
    fn from_request(request: &'a Request<'r>) -> Result<Self, ErrRes> {
        let credentials: Vec<_> = request.headers().get("authorization").collect();
        let credential = match credentials.len() {
            0 => return Err(ErrRes::from("auth: missing header")),
            1 => credentials[0],
            _ => return Err(ErrRes::from("auth: too many headers")),
        };

        let splits = credential.split(" ").collect::<Vec<_>>();
        if splits.len() != 2 || splits[0] != "Bearer" {
            return Err(ErrRes::from("auth: missing Bearer scheme"));
        }
        let encoded = splits[1];

        let decoded: Vec<u8> = decode(encoded)
            .map_err(|err| ErrRes::from(format!("auth: while base64 decoding: {:?}", err)))?;
        let token: Token = serde_json::from_slice(decoded.as_slice())
            .map_err(|err| ErrRes::from(format!("auth: while JSON decoding: {:?}", err)))?;
        Ok(token)
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

#[post("/signup", format = "json", data = "<data>")]
fn signup(data: Json<SignupReq>, store: State<SharedTreeStore>) -> Result<Json<SignupRes>, ErrRes> {
    let mut store = store.lock().expect("can't lock");

    if store.emails.contains_key(&data.email) {
        return Err(ErrRes::from("email exists"));
    }

    let id = TreeStore::make_id(&data.email);
    let user = User::new(&data.password);

    store.emails.insert(data.email.clone(), id);
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

#[post("/login", format = "json", data = "<data>")]
fn login(data: Json<LoginReq>, store: State<SharedTreeStore>) -> Result<Json<LoginRes>, ErrRes> {
    let store = store.lock().expect("can't lock");

    if !store.emails.contains_key(&data.email) {
        return Err(ErrRes::from("no such user"));
    }

    let id = store.emails.get(&data.email).unwrap().clone();
    let user = store.users.get(&id).unwrap();

    if !user.verify_pw(&data.password) {
        return Err(ErrRes::from("password doesn't match"));
    }

    let res = LoginRes {
        token: Token { id },
    };
    Ok(Json(res))
}

#[derive(Serialize)]
struct GetTasksRes {
    tasks: serde_json::Value,
}

#[get("/")]
fn get_tasks(
    req: RequestProxy,
    store: State<SharedTreeStore>,
) -> Result<Json<GetTasksRes>, ErrRes> {
    let token = Token::from_request(req.0)?;
    let store = store.lock().expect("can't lock");

    let id = token.id;

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
    tasks: serde_json::Value,
}

#[post("/", format = "json", data = "<data>")]
fn post_tasks(req: RequestProxy, data: Json<PostTasksReq>, store: State<SharedTreeStore>) -> Result<String, ErrRes> {
    let token = Token::from_request(req.0)?;
    let mut store = store.lock().expect("can't lock");

    let id = token.id;

    if !store.users.contains_key(&id) {
        return Err(ErrRes::from("invalid token"));
    }

    store.tasks.insert(id, data.tasks.clone());

    Ok("".to_string())
}

#[catch(400)]
fn bad_request() -> ErrRes {
    ErrRes::from("bad request")
}

#[catch(401)]
fn unauthorized() -> ErrRes {
    ErrRes::from("requires authentication")
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
            unauthorized,
            not_found,
            unprocessable_entity,
            internal_server_error
        ])
        .manage(Mutex::new(TreeStore::new()))
}

fn main() {
    rocket().launch();
}
