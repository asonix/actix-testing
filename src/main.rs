extern crate actix;
extern crate actix_web;
extern crate futures;

use actix::{Address, ResponseType};
use actix_web::{Application, HttpRequest, HttpServer, Method};

mod actors_actor;
mod main_actor;
mod posts_actor;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

pub struct MissingAddress;

pub struct RequestAddress {
    id: Id,
}

impl ResponseType for RequestAddress {
    type Item = Address<main_actor::MyActor>;
    type Error = MissingAddress;
}

pub struct Post {
    id: Id,
    author: Id,
    content: String,
}

impl ResponseType for Post {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Debug)]
pub struct NewPost {
    author: Id,
    content: String,
}

impl ResponseType for NewPost {
    type Item = Id;
    type Error = ();
}

fn app1(_: HttpRequest) -> &'static str {
    "Hewwo? Mr Obama???"
}

fn app2(_: HttpRequest) -> &'static str {
    "Mr Obama, pwease"
}

fn main() {
    HttpServer::new(|| {
        vec![
            Application::new()
                .prefix("/who")
                .resource("/", |r| r.method(Method::GET).f(app1)),
            Application::new()
                .prefix("/what")
                .resource("/", |r| r.method(Method::GET).f(app2)),
        ]
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
