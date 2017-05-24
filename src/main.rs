extern crate rustyline;
extern crate docopt;

use rustyline::error::ReadlineError;
use docopt::Docopt;

use std::io;
use std::io::prelude::*;
use std::error::Error;

use std::fs::File;
use std::env;
use std::path::Path;

mod sabri;
use sabri::syntax;
use sabri::Sabri;
use sabri::bytecode::Run;

use syntax::{lexer, parser};

use lexer::{BlockTree, process_branch};
use parser::{Traveler, Parser, Expression};

static PROMPT: &'static str = ">> ";

const USAGE: &'static str = "
the glorious sabri language

usage:
    sabri <source>
    sabri repl
    sabri (-h | --help)
    sabri --version
options:
    -h --help   display this message
    --version   display version
";

#[allow(dead_code)]
fn file(path: &str) {
    let mut sabri = Sabri::new();
    let mut runner = Run::new(sabri.env.clone());

    let path = Path::new(path);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("failed to read {}: {}", display,  why.description()),
        Ok(_) => {
            let mut blocks = BlockTree::new(s.as_str(), 0);
            let indents    = blocks.indents();

            let root = blocks.tree(&indents);
            let done = process_branch(&root);

            let traveler = Traveler::new(done);
            let mut parser = Parser::new(traveler);

            match parser.parse() {
                Err(why)  => println!("error: {}", why),
                Ok(stuff) => {
                    match Expression::Block(Box::new(stuff.clone())).compile(&sabri.sym_tab, &mut sabri.bytecode) {
                        Err(why) => println!("error: {}", why),
                        Ok(_)    => {
                            match runner.exec(100_000, &sabri.bytecode.instr, &sabri.bytecode.literals) {
                                Err(e) => println!("{}", e),
                                Ok(()) => (),
                            }
                            sabri.dump_bytecode()
                        },
                    }
                    println!("{:#?}", stuff);                        
                },
            }
        },
    }
}

#[allow(dead_code)]
fn repl() {
    let mut rl = rustyline::Editor::<()>::new();

    let mut sabri = Sabri::new();
    let mut runner = Run::new(sabri.env.clone());

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
                    Ok(stuff) => {
                        match Expression::Block(Box::new(stuff.clone())).compile(&sabri.sym_tab, &mut sabri.bytecode) {
                            Err(why) => println!("error: {}", why),
                            Ok(_)    => {
                                match runner.exec(100_000, &sabri.bytecode.instr, &sabri.bytecode.literals) {
                                    Err(e) => println!("{}", e),
                                    Ok(()) => (),
                                }
                                sabri.dump_bytecode()
                            },
                        }
                        println!("{:#?}", stuff);                        
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
a := 123

putsl("hey", " world")
"#;

    let mut sabri = Sabri::new();

    let mut blocks = BlockTree::new(test, 0);
    let indents    = blocks.indents();

    let root = blocks.tree(&indents);
    let done = process_branch(&root);


    let traveler = Traveler::new(done.clone());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Err(why)  => println!("error: {}", why),
        Ok(stuff) => {
            match Expression::Block(Box::new(stuff.clone())).compile(&sabri.sym_tab, &mut sabri.bytecode) {
                Err(why) => println!("error: {}", why),
                Ok(_)    => {
                    let mut runner = Run::new(sabri.env.clone());
                    match runner.exec(100_000, &sabri.bytecode.instr, &sabri.bytecode.literals) {
                        Err(e) => println!("{}", e),
                        Ok(()) => (),
                    }
                    sabri.dump_bytecode()
                },
            }
            println!("{:#?}", stuff);                        
        },
    }
}

fn main() {
    let argv: Vec<String> = env::args().collect();

    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv.into_iter()).parse())
        .unwrap_or_else(|e| e.exit());

    if args.get_bool("repl") {
        repl()
    } else {
        let source = args.get_str("<source>");

        file(source)
    }
}