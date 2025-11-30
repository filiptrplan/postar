use crate::dsl::{
    ast::*,
    lexer::{Token, process_tokens},
    parser::string_matcher,
};
use chumsky::Parser;

fn main() {
    let file = crate::dsl::File {
        file_name: "test".to_string(),
        contents: "contains \"hello\"".to_string(),
        lexer_spans: None,
    };
    
    let tokens = process_tokens(&file)
        .map_err(|_| ())
        .unwrap()
        .into_iter()
        .map(|(token, _)| token)
        .collect();
    
    println!("Tokens: {:?}", tokens);
    
    let result = string_matcher().parse(&tokens);
    println!("Result: {:?}", result);
    println!("Has output: {}", result.has_output());
    
    if let Some(output) = result.into_output() {
        println!("Output: {:?}", output);
        println!("Output type: {:?}", std::any::type_name_of_val(&output));
    }
}