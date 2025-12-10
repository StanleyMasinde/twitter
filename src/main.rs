use twitter::cli;

#[tokio::main]
async fn main() {
    cli::run().await;
}
