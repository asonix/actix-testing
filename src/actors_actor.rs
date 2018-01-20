use actix::{Actor, Address, AsyncContext, Context, Handler, ResponseType};

use std::collections::HashMap;

use main_actor::MyActor;
use posts_actor::Posts;
use super::{Id, MissingAddress, RequestAddress};

pub struct NewActor;

impl ResponseType for NewActor {
    type Item = Id;
    type Error = ();
}

pub struct Actors {
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

        let actor = MyActor::new(Id(self.actor_id), ctx.address(), self.posts_addr.clone());

        let addr = actor.start();
        self.actors.insert(id, addr);

        Ok(id)
    }
}

impl Handler<RequestAddress> for Actors {
    type Result = Result<Address<MyActor>, MissingAddress>;

    fn handle(&mut self, request_address: RequestAddress, _: &mut Context<Self>) -> Self::Result {
        let RequestAddress { id } = request_address;

        let address = self.actors.get(&id).ok_or(MissingAddress)?.clone();

        Ok(address)
    }
}
