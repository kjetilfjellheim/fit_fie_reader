use clap::{arg, command};
use std::io::{BufReader, BufWriter, Write, Read};
use std::fs::File;

fn main() {
    let argument_matcher: clap::ArgMatches = setup();

    let input = get_input(&argument_matcher);

    let output = get_output(&argument_matcher);

    parse(input, output);

}

fn setup() -> clap::ArgMatches {
    command!() // requires `cargo` feature
        .about("FIT file reader. Parse the results to either file or stdout. Will in future versions allow for filtering.")
        .author("Kjetil Fjellheim <kjetil@forgottendonkey.net>")        
        .arg(
            arg!(
                -i --input <FILE> "Input file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
        )
        .arg(
            arg!(
                -o --output <FILE> "Output file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
        )
        .get_matches()
}

fn get_output(argument_matcher: &clap::ArgMatches) -> Box<dyn Write> {
    let mut output: Box<dyn Write> = Box::new(std::io::stdout());
    if let Some(output_path) = argument_matcher.get_one::<String>("output") {
        let file_result = Box::new(File::create(output_path).unwrap());
        let writer = BufWriter::new(file_result);
        output = Box::new(writer);
    }
    output
}

fn get_input(argument_matcher: &clap::ArgMatches) -> Box<dyn Read> {
    let mut input: Box<dyn Read> = Box::new(std::io::stdin());
    if let Some(input_path) = argument_matcher.get_one::<String>("input") {
        let file_result = Box::new(File::open(input_path).unwrap());
        let reader = BufReader::new(file_result);
        input = Box::new(reader);
    }
    input
}


fn parse(mut input: Box<dyn Read>, mut output: Box<dyn Write>) {
    
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = input.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        output.write(buffer[0..bytes_read].as_ref()).unwrap();
    }
}