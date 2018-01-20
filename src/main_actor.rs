use actix::{Actor, Address, Context, Handler};
use futures::future::Future;

use std::collections::HashSet;
use std::collections::BTreeSet;

use actors_actor::Actors;
use posts_actor::Posts;
use super::{Id, NewPost, Post, RequestAddress, ResponseType};

pub struct Follow {
    id: Id,
}

impl ResponseType for Follow {
    type Item = ();
    type Error = ();
}

pub struct FollowRequest {
    id: Id,
}

impl ResponseType for FollowRequest {
    type Item = ();
    type Error = ();
}

pub struct AcceptRequest {
    id: Id,
}

impl ResponseType for AcceptRequest {
    type Item = ();
    type Error = ();
}

pub struct DenyRequest;

impl ResponseType for DenyRequest {
    type Item = ();
    type Error = ();
}

pub struct MyActor {
    id: Id,
    followers: BTreeSet<Id>,
    following: BTreeSet<Id>,
    posts: Vec<Id>,
    actors_addr: Address<Actors>,
    posts_addr: Address<Posts>,
    requested: HashSet<Id>,
    awaiting: HashSet<Id>,
}

impl MyActor {
    pub fn new(actor_id: Id, actors_addr: Address<Actors>, posts_addr: Address<Posts>) -> Self {
        MyActor {
            id: actor_id,
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            posts: Vec::new(),
            actors_addr: actors_addr,
            posts_addr: posts_addr,
            requested: HashSet::new(),
            awaiting: HashSet::new(),
        }
    }
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
