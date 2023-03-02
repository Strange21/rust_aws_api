use std::{io::{Read, Write}, path::Path, fs};
use aws_sdk_s3::{Client as s3_Client, Region};
use aws_sdk_secretsmanager::{Client};
use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use ssh2::{Session};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};

pub const REGION: &str = "ap-south-1";
pub const BUCKET: &str = "anandbucket2109";

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(|request: Request| {
        handler()
    })).await?;
    
    Ok(())
}

async fn handler() -> Result<Response<Body>, Error>{
    // S3 client setup
    let region_provider = RegionProviderChain::first_try(Region::new(REGION));
    let config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = s3_Client::new(&config);
    let object_key = "rustboot.png".to_string(); // Replace with your S3 object key

    // Read file contents from S3
    let object = s3_client.get_object()
                                .bucket(BUCKET)
                                .key(object_key)
                                .send()
                                .await?;
    let bytes = object.body.collect().await?.into_bytes();

    // Get the secret key. 
    let private_key = get_secret(&config).await.unwrap();
    
    // eprintln!("the private key {}", private_key);
    // EC2 instance setup:
    let host = "ec2-100-25-192-52.compute-1.amazonaws.com".to_string(); // Replace with your EC2 instance IP address
    let username = "ubuntu".to_string(); // Replace with your EC2 instance username
    // let identity = Path::new("ipfs.pem"); // Replace with the path to your private key file
    let path = Path::new("/home/ubuntu/rustboot.png"); // Replace with the destination directory on the EC2 instance

    // Upload file to EC2 instance
    let tcp = std::net::TcpStream::connect(format!("{}:22", host)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_pubkey_memory(&username, None, &private_key, None).unwrap(); // Authenticate with the EC2 instance using public key authentication
    let mut sftp = sess.sftp().unwrap();
    let mut remote_file = sftp.create(path).unwrap();
    remote_file.write_all(&bytes).unwrap(); // Write the file contents to the remote file

    // Upload the file to the IPFS network
    let mut channel = sess.channel_session().unwrap();
    channel.exec("ipfs add /home/ubuntu/rustboot.png").unwrap();// remove the hardcoded value.
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    // println!("{}", s);
    channel.wait_close();
    println!("{}", channel.exit_status().unwrap());

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(format!("CID {}\n\n",s ).to_string().into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn get_secret(config: &SdkConfig) -> Result<String, Box<dyn std::error::Error>> {
    let secret_name = "Aws_test_secrete_key";
    let secret_manager_client = Client::new(config);
    
    let response = secret_manager_client.get_secret_value().secret_id(secret_name).send().await?;
    
    match response.secret_string {
        Some(secret) => Ok(secret),
        None => Err("No secret string found".into())
    }
}