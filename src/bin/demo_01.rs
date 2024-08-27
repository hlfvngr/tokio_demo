use tokio::{fs::File, io::AsyncWriteExt};


#[tokio::main]
async fn main() {
    let mut f = File::create(&"./a.txt").await.unwrap();
    f.write_all(b"Hello, world!").await.unwrap();
    println!("File written successfully");
}