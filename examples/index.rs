use std::collections::HashMap;

use diagnostic_ls::chumsky::{parse, type_inference};

fn main() {
    let source = r#"
    fn test() {
        let a = 3;
        let b = "test";
        0
    }
    
    "#;
    let (ast, errors) = parse(source);
    // if let Some(ast) = ast {
    //     println!("{:#?}", ast);
    // }
    let mut hashmap = HashMap::new();
    if let Some(ast) = ast {
        ast.into_iter().for_each(|(k, v)| {
            type_inference(&v.body, &mut hashmap);
        });
    }
    println!("{:?}", hashmap);
}
