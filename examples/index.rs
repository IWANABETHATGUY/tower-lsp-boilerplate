use diagnostic_ls::chumsky::parse;

fn main() {
    let source = r#"
    fn test() {
        let a = 3;
        let b = 4;
        0
    }
    
    "#;
   let (ast, errors) = parse(source); 
   if let Some(ast) = ast {
       println!("{:#?}", ast);
   }
}