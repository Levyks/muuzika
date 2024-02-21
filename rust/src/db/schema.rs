// @generated automatically by Diesel CLI.

diesel::table! {
    parameters (name) {
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    possible_codes (code) {
        #[max_length = 31]
        code -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    parameters,
    possible_codes,
);
