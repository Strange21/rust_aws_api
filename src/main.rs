use std::{io::{Read, Write}, path::Path, fs};
// use aws_sdk_secretsmanager::model::GetSecretValueRequest;
// use aws_sdk_secretsmanager::SecretsManager;
use rusoto_secretsmanager::{SecretsManager, SecretsManagerClient, GetSecretValueRequest};
// use aws_types::region::Region;
use rusoto_core::{proto::json};
// use rusoto_s3::{S3Client, GetObjectRequest};
// use rusoto_s3::S3;
use aws_sdk_s3::{Client, Region, types::{ByteStream, SdkError}};
use aws_config::meta::region::RegionProviderChain;
use ssh2::{Session, Sftp};
use lambda_runtime::Error;
use tokio_stream::StreamExt;

pub const REGION: &str = "ap-south-1";
pub const BUCKET: &str = "anandbucket2109";

#[tokio::main]
async fn main() -> Result<(), Error> {
    // S3 client setup
    // let s3_client = S3Client::new(Region::UsEast1);
    // let bucket_name = "anandbucket2109".to_string(); // Replace with your S3 bucket name
    let object_key = "rustboot.png".to_string(); // Replace with your S3 object key

    let region_provider = RegionProviderChain::first_try(Region::new(REGION));
    let config = aws_config::from_env().region(region_provider).load().await;
    let s3_client = Client::new(&config);

    let object = s3_client.get_object()
                                .bucket(BUCKET)
                                .key(object_key)
                                .send()
                                .await?;

    let file_path = "./rustboot.png";
    let mut data: ByteStream = object.body;
    let file = fs::File::create(&file_path)?;
    let mut buf_writer = std::io::BufWriter::new(file);

    while let Some(bytes) = data.try_next().await? {
        buf_writer.write(&bytes)?;
    }

    // buf_writer.flush()?;

    // let bytes = object.body.collect().await?.into_bytes();
    // let content = std::str::from_utf8(&bytes)?;

    // eprintln!("data: {:?}", bytes.unwrap().into_bytes());

    // match object {
    //     Ok(output) => {
    //         eprintln!("Go the file");
    //         output.body.collect().await;
    //     }
    //     Err(error) => {
    //         eprintln!("Error retrieving file: {:?}", error);
    //     }
    // };

    // Get the secret key. 
    let private_key = get_secret().await.unwrap();
    
    // eprintln!("the private key {}", private_key);
    // EC2 instance setup:
    let host = "ec2-100-25-192-52.compute-1.amazonaws.com".to_string(); // Replace with your EC2 instance IP address
    let username = "ubuntu".to_string(); // Replace with your EC2 instance username
    // let identity = Path::new("ipfs.pem"); // Replace with the path to your private key file
    let path = Path::new("/home/ubuntu/rustboot.png"); // Replace with the destination directory on the EC2 instance

    // Read file contents from S3
    // let get_request = GetObjectRequest {
    //     bucket: bucket_name.clone(),
    //     key: object_key.clone(),
    //     ..Default::default()
    // };
    // let object = s3_client.get_object(get_request).await.unwrap();
    // let mut content = String::new();
    // let mut buffer = object.body;
    // println!("{}", content);
    
    let content = fs::read_to_string("./rustboot.png")?;

    // Upload file to EC2 instance
    let tcp = std::net::TcpStream::connect(format!("{}:22", host)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_pubkey_memory(&username, None, &private_key, None).unwrap(); // Authenticate with the EC2 instance using public key authentication
    let mut sftp = sess.sftp().unwrap();
    let mut remote_file = sftp.create(path).unwrap();
    remote_file.write_all(content.as_bytes()).unwrap(); // Write the file contents to the remote file

    Ok(())
}

async fn get_secret() -> Result<String, Box<dyn std::error::Error>> {
    let secret_name = "Aws_test_secrete_key";
    // let region_name = "ap-south-1";
    
    let client = SecretsManagerClient::new(rusoto_core::Region::ApSouth1);
    
    let request = GetSecretValueRequest {
        secret_id: secret_name.to_string(),
        ..Default::default()
    };
    
    let response = client.get_secret_value(request).await?;
    
    match response.secret_string {
        Some(secret) => Ok(secret),
        None => Err("No secret string found".into())
    }
}