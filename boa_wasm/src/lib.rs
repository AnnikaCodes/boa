use boa::{Executable, Interpreter, Lexer, Parser, Realm};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    let mut lexer = Lexer::new(src.as_bytes());

    // Goes through and lexes entire given string.
    let mut tokens = Vec::new();

    for token in lexer {
        tokens.push(token.map_err(|e| format!("Lexing Error: {}", e))?);
    }

    let expr = Parser::new(&tokens)
        .parse_all()
        .map_err(|e| JsValue::from(format!("Parsing Error: {}", e)))?;

    // Setup executor
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Setup executor
    expr.run(&mut engine)
        .map_err(|e| JsValue::from(format!("Error: {}", e)))
        .map(|v| v.to_string())
}
