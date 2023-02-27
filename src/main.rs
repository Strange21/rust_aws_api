use std::{io::{Read, Write, BufRead, BufReader}, path::Path, collections::HashMap};
use aws_config::meta::region::RegionProviderChain;
// use aws_sdk_secretsmanager::model::GetSecretValueRequest;
// use aws_sdk_secretsmanager::SecretsManager;
use rusoto_secretsmanager::{SecretsManager, SecretsManagerClient, GetSecretValueRequest};
// use aws_types::region::Region;
use rusoto_core::{Region, credential::EnvironmentProvider, Client};
use rusoto_s3::{S3Client, GetObjectRequest};
use ssh2::{Session, Sftp};
use rusoto_s3::S3;
use lambda_runtime::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // S3 client setup
    let s3_client = S3Client::new(Region::UsEast1);
    let bucket_name = "anandbucket2109".to_string(); // Replace with your S3 bucket name
    let object_key = "rustboot.png".to_string(); // Replace with your S3 object key

    // Get the secret key. 
    let private_key = get_secret().await.unwrap();

    // EC2 instance setup
    let host = "ec2-100-25-192-52.compute-1.amazonaws.com".to_string(); // Replace with your EC2 instance IP address
    let username = "ubuntu".to_string(); // Replace with your EC2 instance username
    // let identity = Path::new("ipfs.pem"); // Replace with the path to your private key file
    let path = Path::new("/home/ubuntu/rustboot.png"); // Replace with the destination directory on the EC2 instance

    // Read file contents from S3
    let get_request = GetObjectRequest {
        bucket: bucket_name.clone(),
        key: object_key.clone(),
        ..Default::default()
    };
    let object = s3_client.get_object(get_request).await.unwrap();
    let mut content = String::new();
    object.body.unwrap().into_blocking_read().read_to_string(&mut content).unwrap();
    // println!("{}", content);
    
    // let content = fs::read_to_string("./myfile.txt")?;

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
    let region_name = "ap-south-1";
    
    let client = SecretsManagerClient::new(Region::ApSouth1);
    
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