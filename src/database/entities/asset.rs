//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

use crate::database::enums::AssetSchema;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "asset"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq)]
pub struct Model {
    pub idx: i32,
    pub media_idx: Option<i32>,
    pub id: String,
    pub schema: AssetSchema,
    pub added_at: i64,
    pub details: Option<String>,
    pub issued_supply: String,
    pub name: String,
    pub precision: u8,
    pub ticker: Option<String>,
    pub timestamp: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Idx,
    MediaIdx,
    Id,
    Schema,
    AddedAt,
    Details,
    IssuedSupply,
    Name,
    Precision,
    Ticker,
    Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Idx,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i32;
    fn auto_increment() -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AssetTransfer,
    Media,
    Token,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Idx => ColumnType::Integer.def(),
            Self::MediaIdx => ColumnType::Integer.def().null(),
            Self::Id => ColumnType::String(StringLen::None).def().unique(),
            Self::Schema => ColumnType::SmallInteger.def(),
            Self::AddedAt => ColumnType::BigInteger.def(),
            Self::Details => ColumnType::String(StringLen::None).def().null(),
            Self::IssuedSupply => ColumnType::String(StringLen::None).def(),
            Self::Name => ColumnType::String(StringLen::None).def(),
            Self::Precision => ColumnType::SmallInteger.def(),
            Self::Ticker => ColumnType::String(StringLen::None).def().null(),
            Self::Timestamp => ColumnType::BigInteger.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AssetTransfer => Entity::has_many(super::asset_transfer::Entity).into(),
            Self::Media => Entity::belongs_to(super::media::Entity)
                .from(Column::MediaIdx)
                .to(super::media::Column::Idx)
                .into(),
            Self::Token => Entity::has_many(super::token::Entity).into(),
        }
    }
}

impl Related<super::asset_transfer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AssetTransfer.def()
    }
}

impl Related<super::media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl Related<super::token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Token.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
