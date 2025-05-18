#[macro_export]
macro_rules! handle_response {
    ($generic_response:expr, $expected_response_variant:path) => {
        match $generic_response {
            Ok(generic_response) => match generic_response.response {
                Some($expected_response_variant(response)) => match response.response {
                    Some(response) => Ok(response),
                    None => Err(crate::errors::RequestError::EmptyResponse),
                },
                Some(_) => Err(crate::errors::RequestError::UnexpectedResponse),
                None => Err(crate::errors::RequestError::EmptyResponse),
            },
            Err(_) => Err(crate::errors::RequestError::FailedToReceive),
        }
    };
}

#[macro_export]
macro_rules! handle_response_with_oneof_error {
    ($generic_response:expr, $expected_response_variant:path, $success_variant:path, $error_variant:path) => {
        match $crate::handle_response!($generic_response, $expected_response_variant) {
            Ok(specific_response) => match specific_response {
                $success_variant(success) => Ok(success),
                $error_variant(error) => match error.error {
                    Some(error) => Err(error.into()),
                    None => Err($crate::errors::RequestError::EmptyResponse),
                }
            },
            Err(e) => Err(e),
        }
    };
}