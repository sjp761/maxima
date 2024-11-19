use crate::proto_struct;

proto_struct!(User, {
    account_id: String,
    persona_id: String,
    display_name: String,
    unique_name: String,
    nickname: String,
});
