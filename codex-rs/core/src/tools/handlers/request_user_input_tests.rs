use super::*;
use pretty_assertions::assert_eq;

#[test]
fn request_user_input_tool_description_is_unconditional() {
    assert_eq!(
        request_user_input_tool_description(),
        "Request user input for one to three short questions and wait for the response."
    );
}
