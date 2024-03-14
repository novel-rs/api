use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "text")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub identifier: String,
    pub date_time: Option<NaiveDateTime>,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
