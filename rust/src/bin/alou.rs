use muuzika::proto::job::Job;
use muuzika::proto::Log;

pub fn main() {
    let job = Job::Log(Log {
        message: "Hello, world!".to_string(),
    });

    let mut buf = Vec::new();
    job.encode(&mut buf);

    println!("{:?}", buf);

    let hex_string = buf.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    println!("{:?}", hex_string);
}
