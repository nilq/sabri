extern crate rustyline;

use rustyline::error::ReadlineError;

mod sabri;
use sabri::syntax;
use sabri::Sabri;
use sabri::bytecode::Run;

use syntax::{lexer, parser};

use lexer::{BlockTree, process_branch};
use parser::{Traveler, Parser, Statement, Expression};

static PROMPT: &'static str = ">> ";

#[allow(dead_code)]
fn repl() {
    let mut rl = rustyline::Editor::<()>::new();

    let mut sabri = Sabri::new();

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
                        match e.clone() {
                            Statement::Expression(ex) => match *ex {
                                Expression::EOF => (),
                                _ => match ex.compile(&sabri.sym_tab, &mut sabri.bytecode) {
                                        Err(why) => println!("error: {}", why),
                                        Ok(_)    => {
                                            let mut runner = Run::new(sabri.env.clone());
                                            match runner.exec(100_000, &sabri.bytecode.instr, &sabri.bytecode.literals) {
                                                Err(e) => println!("{}", e),
                                                Ok(()) => (),
                                            }

                                            sabri.dump_bytecode()
                                        },
                                    },
                            },
                            _ => (),
                        }
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