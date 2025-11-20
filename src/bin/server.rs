use sample_guard::api::start_server;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");
    
    println!("SampleGuard API Server");
    println!("======================");
    println!("Starting server on http://{}:{}", host, port);
    println!("API endpoints available at: http://{}:{}/api/v1", host, port);
    println!();
    
    start_server(&host, port).await
}

