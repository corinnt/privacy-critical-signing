
pub fn private_fmt(private_data: String) -> String {
    let secret = format!("this dat is still private: {}", private_data); 
    println!("{}", secret); 
    secret
}