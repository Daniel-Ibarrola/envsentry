fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    let environment = std::env::var("ENVIRONMENT").unwrap();
    let secret = std::env::var("SECRET_KEY").unwrap();
    println!("The secret is: {}", secret);
}