use std::io::{Read, Write};
use rusoto_core::{Region, credential::EnvironmentProvider};
use rusoto_s3::{S3Client, GetObjectRequest};
use ssh2::{Session, Sftp};
use rusoto_core::encoding::ContentEncoding::Identity;
use rusoto_s3::S3;

fn main() {
    // S3 client setup
    let s3_client = S3Client::new(Region::UsEast1);
    let bucket_name = "my-bucket".to_string(); // Replace with your S3 bucket name
    let object_key = "my-file.txt".to_string(); // Replace with your S3 object key

    // EC2 instance setup
    let host = "ec2-instance-ip".to_string(); // Replace with your EC2 instance IP address
    let username = "ec2-user".to_string(); // Replace with your EC2 instance username
    let identity = ssh2::Identity::from_file("/path/to/private-key", None).unwrap(); // Replace with the path to your private key file
    let path = "/path/to/destination/directory".to_string(); // Replace with the destination directory on the EC2 instance

    // Read file contents from S3
    let get_request = GetObjectRequest {
        bucket: bucket_name.clone(),
        key: object_key.clone(),
        ..Default::default()
    };
    let object = s3_client.get_object(get_request).sync().unwrap();
    let mut content = String::new();
    object.body.unwrap().into_blocking_read().read_to_string(&mut content).unwrap();

    // Upload file to EC2 instance
    let tcp = std::net::TcpStream::connect(format!("{}:22", host)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp).unwrap();
    sess.userauth_pubkey_memory(&username, None, &identity, None).unwrap(); // Authenticate with the EC2 instance using public key authentication
    let mut sftp = sess.sftp().unwrap();
    let mut remote_file = sftp.create(&format!("{}/{}", path, object_key)).unwrap();
    remote_file.write_all(content.as_bytes()).unwrap(); // Write the file contents to the remote file
}
