extern crate rustyline;

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::Editor;

mod sabri;
use sabri::syntax;

use syntax::lexer;

use lexer::{BlockTree, process_branch};

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

                println!("{:#?}", done)
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