use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config};
use l_lang::generated::{NodeRef, Parser};
use l_lang::parser::{tokenize, Token};
use l_lang::syntax_tree::init_syntax_tree;
use logos::Logos;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        std::process::exit(1);
    }

    let source = std::fs::read_to_string(&args[1])?;
    let ast = args.get(2).is_some();
    let mut diags = vec![];
    let (tokens, ranges) = tokenize(Token::lexer(&source), &mut diags);
    let cst = Parser::parse(&source, tokens, ranges, &mut diags);
    if ast {
        let ret = init_syntax_tree(&cst);
        dbg!(&ret);
    } else {
        println!("{cst}");
    }

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = Config::default();
    let file = SimpleFile::new(&args[1], &source);
    for diag in diags.iter() {
        term::emit(&mut writer.lock(), &config, &file, diag).unwrap();
    }
    Ok(())
}
