extern crate rustyline;

use rustyline::error::ReadlineError;

mod sabri;
use sabri::syntax;

use syntax::{lexer, parser};

use lexer::{BlockTree, process_branch};
use parser::{Traveler, Parser};

static PROMPT: &'static str = ">> ";

#[allow(dead_code)]
fn repl() {
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                let mut blocks = BlockTree::new(line.as_str(), 0);
                let indents    = blocks.indents();

                let root = blocks.tree(&indents);
                let done = process_branch(&root);

                let traveler = Traveler::new(done);
                let mut parser = Parser::new(traveler);

                match parser.parse() {
                    Err(why)  => println!("error: {}", why),
                    Ok(stuff) => for e in stuff {
                        println!("{:#?}", e)
                    },
                }
            }

            Err(ReadlineError::Interrupted) => {
                println!("interrupted");
                break
            }

            Err(ReadlineError::Eof) => {
                println!("eof");
                break
            }

            Err(err) => {
                println!("error: {:?}", err);
                break
            }
        }
    }
}

#[allow(dead_code)]
fn test() {
    let test = r#"
~ a comment
greet := |a|
  print("yo, " + a)

hello_world = | greet("world")
"#;

    let mut blocks = BlockTree::new(test, 0);
    let indents    = blocks.indents();

    let root = blocks.tree(&indents);
    let done = process_branch(&root);



    println!("{:#?}", done)
}

fn main() {
    repl()
}