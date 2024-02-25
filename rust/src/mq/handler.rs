use std::ops::ControlFlow;

use diesel::IntoSql;
use fe2o3_amqp::Delivery;
use fe2o3_amqp::link::delivery::DeliveryInfo;
use fe2o3_amqp::link::receiver::TerminalDeliveryState;
use fe2o3_amqp::types::definitions::{AmqpError, Error, ErrorCondition};
use fe2o3_amqp::types::messaging::{
    Accepted, AmqpValue, FromBody, FromEmptyBody, IntoBody, Rejected,
};
use fe2o3_amqp::types::primitives::Value;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::errors::MuuzikaResult;

#[derive(Debug, Deserialize, Serialize)]
pub enum Job {
    Log(String),
}

impl IntoBody for Job {
    type Body = AmqpValue<Self>;

    fn into_body(self) -> Self::Body {
        AmqpValue(self)
    }
}

impl FromEmptyBody for Job {}

impl<'de> FromBody<'de> for Job {
    type Body = AmqpValue<Self>;

    fn from_body(body: Self::Body) -> Self {
        body.0
    }
}

pub async fn process_job(job: Job) -> MuuzikaResult<()> {
    match job {
        Job::Log(message) => {
            info!("Received message: {}", message);
        }
    }

    Ok(())
}

pub fn handle_mq_message(
    delivery: Delivery<Value>,
    tx: &UnboundedSender<(DeliveryInfo, TerminalDeliveryState)>,
) -> Result<(), (DeliveryInfo, Option<Error>)> {
    let (info, message) = delivery.into_parts();

    info!("iai meu nobre {:?}", message.body);

    let job = match serde_amqp::from_value(message.body) {
        Ok(job) => job,
        Err(e) => {
            error!(
                "Error deserializing message with delivery id: {}, {:?}",
                info.delivery_id(),
                e
            );
            let condition = ErrorCondition::AmqpError(AmqpError::DecodeError);
            return Err((
                info,
                Some(Error {
                    condition,
                    description: Some(e.to_string()),
                    info: None,
                }),
            ));
        }
    };

    let tx = tx.clone();
    tokio::spawn(async move {
        let state = match process_job(job).await {
            Ok(_) => TerminalDeliveryState::Accepted(Accepted {}),
            Err(e) => {
                error!("Error processing message: {:?}", e);
                TerminalDeliveryState::Rejected(Rejected {
                    error: Some(e.into()),
                })
            }
        };

        tx.send((info, state)).unwrap();
    });

    Ok(())
}
