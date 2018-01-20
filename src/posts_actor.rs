use actix::{Actor, Context, Handler};

use std::collections::HashMap;

use super::{Id, NewPost, Post};

pub struct Posts {
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
        let NewPost { author, content } = new_post;

        let id = Id(self.post_id);
        self.post_id += 1;

        self.posts.insert(
            id,
            Post {
                id: id,
                author: author,
                content: content,
            },
        );

        Ok(id)
    }
}
