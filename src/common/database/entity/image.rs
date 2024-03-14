use sea_orm::entity::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "image")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub url: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
