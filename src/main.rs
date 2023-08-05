extern crate queues;

use clap::{arg, command};
use std::io::{BufReader, BufWriter, Write, Read };
use std::io::prelude::*;
use std::fs::File;
use regex::Regex;
use queues::CircularBuffer;

struct Arguments<'a> {
    input: Box<dyn Read>,
    output: Box<dyn Write>,
    regexp: Vec<String>,
    before_lines: &'a i32,
    after_lines: &'a i32
}

fn main() {
    let argument_matcher: clap::ArgMatches = setup();

    let arguments = get_arguments(argument_matcher);

    parse(arguments);

}

fn setup() -> clap::ArgMatches {
    command!() // requires `cargo` feature
        .about("FIT file reader. Parse the results to either file or stdout. Will in future versions allow for filtering.")
        .author("Kjetil Fjellheim <kjetil@forgottendonkey.net>")        
        .arg(
            arg!(
                -i --input <FILE> "Input file"
            )                
            .required(false)
        )
        .arg(
            arg!(
                -o --output <FILE> "Output file"
            )            
            .required(false)
        )
        .arg(
            arg!(
                -r --regexp <REGEXP> "Regular expression to filter on"
            )            
            .value_parser(clap::value_parser!(String))
            .required(false)
        )   
        .arg(
            arg!(
                -b --before <NUMN> "Number of lines to include before"
            )            
            .value_parser(clap::value_parser!(i32))
            .default_missing_value("0")
            .required(false)
        )         
        .arg(
            arg!(
                -a --after <NUMN> "Number of lines to include after"
            )            
            .value_parser(clap::value_parser!(i32))
            .default_missing_value("0")
            .required(false)
        )                     
        .get_matches()
}

fn get_arguments<'a>(argument_matcher: clap::ArgMatches) -> Arguments<'a> {
    let input: Box<dyn Read> = get_input(&argument_matcher);
    let output: Box<dyn Write> = get_output(&argument_matcher);
    let regxp: Vec<String> = get_regexp(&argument_matcher);
    let before_lines: &i32 = get_argument_value(&argument_matcher, "before");
    let after_lines: &i32 = get_argument_value(&argument_matcher, "after");

    Arguments {
        input: input,
        output: output,
        regexp: regxp,
        before_lines: before_lines,
        after_lines: after_lines
    }
}

fn get_regexp(argument_matcher: &clap::ArgMatches) -> Vec<String> {
    argument_matcher.get_many::<String>("regexp")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
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

fn get_argument_value<'a, T: Clone + std::marker::Sync + std::marker::Send>(argument_matcher: &clap::ArgMatches, argument_name: &str) -> &'static T {
    argument_matcher.get_one::<&'static T>(argument_name).expect("Invalid argument")
}

fn parse(arguments: Arguments) {
    let input = arguments.input;
    let mut output = arguments.output;
    let reader = BufReader::new(input);
    let regexps = arguments.regexp.into_iter().map(|r| Regex::new(&r).expect("Invalid regular expression")).collect::<Vec<Regex>>();
    for line in reader.lines() {        
        match line {
            Ok(line) => {               
                if is_match_any(&line, &regexps) {
                    output.write_all(&line.as_bytes()).unwrap();
                    output.write_all(b"\n").unwrap();
                }               
            },
            Err(e) => {
                std::io::stderr().write_all(format!("Error reading line: {}", e).as_bytes()).unwrap();
            }
        }                
    }
}

fn is_match_any(line: &String, regexps: &Vec<Regex>) -> bool {
    for regexp in regexps {
        if regexp.is_match(&line) {
            return true;
        }
    }
    false
}
