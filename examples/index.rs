use anyhow::Ok;
use nrs_language_server::{
    nrs_lang::{parse, ParserResult},
    semantic_analyze,
};

fn main() -> anyhow::Result<()> {
    let source = include_str!("./basic.nrs");
    // let source = r#"
    // test
    // println!("{:?}", &source[10..11]);
    let ParserResult {
        ast,
        parse_errors,
        semantic_tokens: _,
    } = parse(source);
    println!("{parse_errors:?}");
    let ast = if let Some(ref ast) = ast {
        println!("{ast:#?}");
        ast
    } else {
        println!("{parse_errors:?}");
        return Ok(());
    };
    let table = semantic_analyze::analyze_program(ast)?;
    dbg!(&table);
    Ok(())
}
