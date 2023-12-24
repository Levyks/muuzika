mod proto;

use std::net::SocketAddr;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::proto::{SingleUintRequest, SingleUint64Response, SingleStringResponse, TwoStringsRequest};
use crate::proto::calculator_server::{Calculator, CalculatorServer};
use crate::proto::string_manipulator_server::{StringManipulator, StringManipulatorServer};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port = args[1].parse::<u16>().unwrap();
    let source = args[2].clone();

    let addr = SocketAddr::from(([127,0,0,1], port));
    let calculator = CalculatorImpl::new(source.clone());
    let string_manipulator = StringManipulatorImpl::new(source.clone());

    Server::builder()
        .add_service(CalculatorServer::new(calculator))
        .add_service(StringManipulatorServer::new(string_manipulator))
        .serve(addr)
        .await?;

    Ok(())
}
