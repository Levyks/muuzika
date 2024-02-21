use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::parameters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Parameter {
    pub name: String,
    pub value: String,
}