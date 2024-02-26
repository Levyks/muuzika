use std::env;

use fe2o3_amqp::link::receiver::TerminalDeliveryState;
use fe2o3_amqp::types::messaging::{Accepted, Message};

use muuzika::errors::MuuzikaResult;
use muuzika::mq::utils::{listen, MqParameters};
use muuzika::state::read_env_var_or_else;

#[tokio::main]
pub async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();

    let parameters = MqParameters::from_env();
    let address = read_env_var_or_else("MQ_JOBS_ADDRESS", || "muuzika_jobs".to_string());

    listen(parameters, address, handle_message).await.unwrap();
}

async fn handle_message(msg: Message<String>) -> MuuzikaResult<TerminalDeliveryState> {
    println!("IAIII MEU NOBRE: {:?}", msg.body);

    Ok(Accepted {}.into())
}
