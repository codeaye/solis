use crate::utils::{input, read_file};
use interpreter::Interpreter;
use lexer::Lexer;
use log::{debug, error};
use parser::Parser;
use std::{path::PathBuf, rc::Rc};

pub fn run(file_path: PathBuf) {
    let code = read_file(&file_path);
    let Ok(code) = code else {
        return error!("Could not read file at path {:?}", file_path);
    };

    run_code(code, Rc::new(Interpreter::new()))
}

pub fn repl() {
    let interpreter = Rc::new(Interpreter::new());
    loop {
        let code = input("> ").unwrap();
        let code = code.replace("\\n", "\n");

        if code == ":exit" {
            break;
        }

        run_code(code, interpreter.clone())
    }
}

pub fn run_code(code: String, interpreter: Rc<Interpreter>) {
    let tokens_raw = Lexer::lex(code.as_str());
    let Ok(tokens) = tokens_raw else {
        return error!("{}", tokens_raw.err().unwrap());
    };

    let parsed = Parser::new(tokens).parse();
    let Ok(parsed_expr) = parsed else {
        return error!("{}", parsed.err().unwrap());
    };

    debug!("Parsed code");

    let interpreted_r = interpreter.interpret(parsed_expr);

    let Ok(_interpreter) = interpreted_r else {
        return error!("{}", interpreted_r.err().unwrap())
    };
}
