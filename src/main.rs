use std::error::Error;
use std::sync::Arc;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use rune::termcolor::{ColorChoice, StandardStream};
use rune::EmitDiagnostics as _;
use runestick::{Context, Source, Value, Vm};

fn format_code(source: &str) -> Source {
    let source = format!("fn repl_main__() {{\n{}\n}}", source);
    Source::new("script", source)
}

fn handle_line(
    context: Arc<Context>,
    source: &mut String,
    line: &str,
) -> Result<Value, Box<dyn Error>> {
    if line == ".clear" {
        source.clear();
        return Ok(Value::Unit);
    }

    if line == ".show" {
        println!("{}", source);
        return Ok(Value::Unit);
    }

    if line == ".help" {
        println!("runepl - A REPL for Rune.");
        println!();
        println!("Available commands:");
        println!(" .show  -- Show the current full code buffer.");
        println!(" .clear -- Clear the current full code buffer.");
        println!(" .help  -- Show this help.");
        println!();
        println!("Everything else is appended as code into the buffer and executed.");
        return Ok(Value::Unit);
    }

    let options = rune::Options::default();
    let mut warnings = rune::Warnings::new();

    let mut new_source = source.clone();
    new_source.push_str("\n");
    new_source.push_str(line);
    let full_source = format_code(&new_source);

    let unit = match rune::load_source(&*context, &options, full_source, &mut warnings) {
        Ok(unit) => unit,
        Err(error) => {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            error.emit_diagnostics(&mut writer)?;
            return Ok(Value::Unit);
        }
    };

    if !warnings.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        rune::emit_warning_diagnostics(&mut writer, &warnings, &unit)?;
    }

    let vm = Vm::new(context.clone(), Arc::new(unit));
    let mut execution = vm.call(&["repl_main__"], ())?;
    let value = execution.complete()?;

    source.push_str("\n");
    source.push_str(line);

    Ok(value)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new();

    let context = Arc::new(rune::default_context()?);
    let mut source = String::new();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let result = handle_line(Arc::clone(&context), &mut source, &line)?;
                println!("=> {:?}", result);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")?;

    Ok(())
}
