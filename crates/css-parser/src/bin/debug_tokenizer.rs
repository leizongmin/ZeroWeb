use zero_css_parser::tokenizer::Tokenizer;

fn main() {
    let tokens: Vec<_> = Tokenizer::new("").map(|s| s.token).collect();
    println!("Empty string tokens: {:?}", tokens);
    println!("Last token: {:?}", tokens.last());

    let tokens2: Vec<_> = Tokenizer::new("a").map(|s| s.token).collect();
    println!("Single char tokens: {:?}", tokens2);
    println!("Last token: {:?}", tokens2.last());
}
