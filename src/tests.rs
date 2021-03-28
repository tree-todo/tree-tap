use crate::rocket;
use rocket::http::{ContentType, Status};
use rocket::local::Client;

/*
fn register_hit(client: &Client) {
    let response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

fn get_count(client: &Client) -> usize {
    let mut response = client.get("/count").dispatch();
    response.body_string().and_then(|s| s.parse().ok()).unwrap()
}

#[test]
fn test_count() {
    let client = Client::new(super::rocket()).unwrap();

    // Count should start at 0.
    assert_eq!(get_count(&client), 0);

    for _ in 0..99 {
        register_hit(&client);
    }
    assert_eq!(get_count(&client), 99);

    register_hit(&client);
    assert_eq!(get_count(&client), 100);
}

#[test]
fn test_raw_state_count() {
    use super::{count, index};
    use rocket::State;

    let rocket = super::rocket();

    assert_eq!(count(State::from(&rocket).unwrap()), "0");
    assert!(index(State::from(&rocket).unwrap()).0.contains("Visits: 1"));
    assert_eq!(count(State::from(&rocket).unwrap()), "1");
}
*/

#[test]
fn test_login() {
    let client = Client::new(rocket()).unwrap();

    let mut res = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{ "email": "a@a.com", "password": "p" }"#)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);

    let body = res.body_string().unwrap();
    assert!(body.contains("token"));

    let res = client.post("/login").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), Status::BadRequest);

    let res = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{ }"#)
        .dispatch();
    assert_eq!(res.status(), Status::UnprocessableEntity);

    let res = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{ "email": 10, "password: "" }"#)
        .dispatch();
    assert_eq!(res.status(), Status::UnprocessableEntity);
}
