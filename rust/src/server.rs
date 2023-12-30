

use std::net::SocketAddr;
use std::pin::Pin;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use muuzika::proto::{SingleUintRequest, SingleUint64Response, SingleStringResponse, TwoStringsRequest};
use muuzika::proto::calculator_server::{Calculator, CalculatorServer};
use muuzika::proto::string_manipulator_server::{StringManipulator, StringManipulatorServer};
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::codegen::tokio_stream::Stream;
use muuzika::lobby::service::LobbyServiceImpl;
use muuzika::proto::lobby_service_server::LobbyServiceServer;

fn fib(n: u32) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    let mut a = 0;
    let mut b = 1;
    let mut c = 0;
    for _ in 2..=n {
        c = a + b;
        a = b;
        b = c;
    }
    return c;
}

fn fact(n: u32) -> u64 {
    if n <= 1 {
        return 1
    }
    let mut result = 1u64;
    for i in 2..=n {
        result *= i as u64;
    }
    return result;
}



struct CalculatorImpl {
    source: String
}

impl CalculatorImpl {
    fn new(source: String) -> Self {
        Self {
            source
        }
    }
}

#[tonic::async_trait]
impl Calculator for CalculatorImpl {
    async fn fibonacci(&self, request: Request<SingleUintRequest>) -> Result<Response<SingleUint64Response>, Status> {
        let result = fib(request.into_inner().number);
        Ok(Response::new(SingleUint64Response {
            result,
            source: self.source.clone()
        }))
    }

    async fn factorial(&self, request: Request<SingleUintRequest>) -> Result<Response<SingleUint64Response>, Status> {
        let result = fact(request.into_inner().number);
        Ok(Response::new(SingleUint64Response {
            result,
            source: self.source.clone()
        }))
    }
}

struct StringManipulatorImpl {
    source: String
}

impl StringManipulatorImpl {
    fn new(source: String) -> Self {
        Self {
            source
        }
    }
}

#[tonic::async_trait]
impl StringManipulator for StringManipulatorImpl {
    async fn concatenate(&self, request: Request<TwoStringsRequest>) -> Result<Response<SingleStringResponse>, Status> {
        let request = request.into_inner();
        let result = format!("{}{}", request.first, request.second);
        Ok(Response::new(SingleStringResponse {
            result,
            source: self.source.clone()
        }))
    }

    type ConcatenateStreamStream = Pin<Box<dyn Stream<Item = Result<SingleStringResponse, Status>> + Send>>;

    async fn concatenate_stream(&self, request: Request<TwoStringsRequest>) -> Result<Response<Self::ConcatenateStreamStream>, Status> {
        let (tx, rx) = mpsc::unbounded_channel();

        let request = request.into_inner();
        let source = self.source.clone();

        tokio::spawn(async move {
            let mut current = request.first;

            for c in request.second.chars() {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                current.push(c);
                if let Err(_) = tx.send(Ok(SingleStringResponse {
                    result: current.clone(),
                    source: source.clone()
                })) {
                    break;
                }
            }
        });

        let stream = UnboundedReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::ConcatenateStreamStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port = args[1].parse::<u16>().unwrap();
    let source = args[2].clone();

    println!("Starting server on port {} with source {}", port, source);

    let addr = SocketAddr::from(([127,0,0,1], port));

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder(config).build()?;

    let lobby_service = LobbyServiceImpl::new(pool.clone());
    let calculator = CalculatorImpl::new(source.clone());
    let string_manipulator = StringManipulatorImpl::new(source.clone());

    Server::builder()
        .add_service(LobbyServiceServer::new(lobby_service))
        .add_service(CalculatorServer::new(calculator))
        .add_service(StringManipulatorServer::new(string_manipulator))
        .serve(addr)
        .await?;

    Ok(())
}
