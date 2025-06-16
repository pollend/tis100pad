use std::marker::PhantomData;

use sqlx::{Pool, Postgres};

pub struct Repository<'a, DB> {
    _type: PhantomData<&'a DB>,
}

impl From<&Pool<Postgres>> for Repository<'static, Postgres> {
    fn from(_: &Pool<Postgres>) -> Self {
        Repository::<'static, Postgres> { _type: PhantomData }
    }
}

