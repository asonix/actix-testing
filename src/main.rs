extern crate actix;
extern crate actix_web;
extern crate futures;

use actix::{Actor, Address, AsyncContext, Context, Handler, ResponseType};
use actix_web::{Application, HttpRequest, HttpServer, Method};
use futures::Future;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Id(usize);

struct RequestAddress {
    id: Id,
}

impl ResponseType for RequestAddress {
    type Item = Address<MyActor>;
    type Error = MissingAddress;
}

struct MissingAddress;

struct Post {
    id: Id,
    author: Id,
    content: String,
}

impl ResponseType for Post {
    type Item = ();
    type Error = ();
}

#[derive(Clone, Debug)]
struct NewPost {
    author: Id,
    content: String,
}

impl ResponseType for NewPost {
    type Item = Id;
    type Error = ();
}

struct Posts {
    post_id: usize,
    posts: HashMap<Id, Post>,
}

impl Default for Posts {
    fn default() -> Self {
        Posts {
            post_id: 0,
            posts: HashMap::new(),
        }
    }
}

impl Actor for Posts {
    type Context = Context<Self>;
}

impl Handler<NewPost> for Posts {
    type Result = Result<Id, ()>;

    fn handle(&mut self, new_post: NewPost, _: &mut Context<Self>) -> Self::Result {
        let NewPost {
            author,
            content,
        } = new_post;

        let id = Id(self.post_id);
        self.post_id += 1;

        self.posts.insert(id, Post {
            id: id,
            author: author,
            content: content,
        });

        Ok(id)
    }
}

struct NewActor;

impl ResponseType for NewActor {
    type Item = Id;
    type Error = ();
}


struct Actors {
    actor_id: usize,
    actors: HashMap<Id, Address<MyActor>>,
    posts_addr: Address<Posts>,
}

impl Actor for Actors {
    type Context = Context<Self>;
}

impl Handler<NewActor> for Actors {
    type Result = Result<Id, ()>;

    fn handle(&mut self, _: NewActor, ctx: &mut Context<Self>) -> Self::Result {
        let id = Id(self.actor_id);
        self.actor_id += 1;

        let actor = MyActor {
            id: Id(self.actor_id),
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            posts: Vec::new(),
            actors_addr: ctx.address(),
            posts_addr: self.posts_addr.clone(),
            requested: HashSet::new(),
            awaiting: HashSet::new(),
        };

        let addr = actor.start();
        self.actors.insert(id, addr);

        Ok(id)
    }
}

impl Handler<RequestAddress> for Actors {
    type Result = Result<Address<MyActor>, MissingAddress>;

    fn handle(&mut self, request_address: RequestAddress, _: &mut Context<Self>) -> Self::Result {
        let RequestAddress { id } = request_address;

        let address = self.actors
            .get(&id)
            .ok_or(MissingAddress)?
            .clone();

        Ok(address)
    }
}

struct Follow {
    id: Id,
}

impl ResponseType for Follow {
    type Item = ();
    type Error = ();
}

struct FollowRequest {
    id: Id,
}

impl ResponseType for FollowRequest {
    type Item = ();
    type Error = ();
}

struct AcceptRequest {
    id: Id,
}

impl ResponseType for AcceptRequest {
    type Item = ();
    type Error = ();
}

struct DenyRequest;

impl ResponseType for DenyRequest {
    type Item = ();
    type Error = ();
}

struct MyActor {
    id: Id,
    followers: BTreeSet<Id>,
    following: BTreeSet<Id>,
    posts: Vec<Id>,
    actors_addr: Address<Actors>,
    posts_addr: Address<Posts>,
    requested: HashSet<Id>,
    awaiting: HashSet<Id>,
}

impl Actor for MyActor {
    type Context = Context<Self>;
}

impl Handler<Post> for MyActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, post: Post, _: &mut Context<Self>) -> Self::Result {
        self.posts.push(post.id);

        Ok(())
    }
}

impl Handler<NewPost> for MyActor {
    type Result = Result<Id, ()>;

    fn handle(&mut self, new_post: NewPost, _: &mut Context<Self>) -> Self::Result {
        let id = self.posts_addr.call_fut(new_post.clone()).wait().map_err(|_| ())??;

        self.posts.push(id);

        for id in &self.followers {
            let actor = self.actors_addr.call_fut(RequestAddress { id: id.clone() }).map_err(|_| ()).wait()?.map_err(|_| ())?;
            actor.send(new_post.clone());
        }

        Ok(id)
    }
}

impl Handler<Follow> for MyActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, follow: Follow, _: &mut Context<Self>) -> Self::Result {
        let actor = self.actors_addr.call_fut(RequestAddress { id: follow.id }).map_err(|_| ()).wait()?.map_err(|_| ())?;

        actor.call_fut(FollowRequest { id: self.id }).map_err(|_| ()).wait()?.map_err(|_| ())?;
        self.awaiting.insert(follow.id);

        Ok(())
    }
}

impl Handler<FollowRequest> for MyActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, follow_request: FollowRequest, _: &mut Context<Self>) -> Self::Result {
        self.requested.insert(follow_request.id);

        Ok(())
    }
}

impl Handler<AcceptRequest> for MyActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, accept_request: AcceptRequest, _: &mut Context<Self>) -> Self::Result {
        if self.awaiting.remove(&accept_request.id) {
            self.following.insert(accept_request.id);
        }

        Ok(())
    }
}

impl Handler<DenyRequest> for MyActor {
    type Result = Result<(), ()>;

    fn handle(&mut self, _: DenyRequest, _: &mut Context<Self>) -> Self::Result {
        Ok(())
    }
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
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}
