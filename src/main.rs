extern crate queues;

use clap::{arg, command};
use queues::*;
use regex::Regex;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::{BufReader, BufWriter, Read, Write};

struct Arguments {
    input: Box<dyn Read>,
    output: Box<dyn Write>,
    regexp: Vec<String>,
    before_lines: i32,
    after_lines: i32,
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
            .action(clap::ArgAction::Append)
            .value_parser(clap::value_parser!(String))
            .required(false)
        )
        .arg(
            arg!(
                -b --before <NUM> "Number of lines to include before"
            )
            .value_parser(clap::value_parser!(i32))
            .default_missing_value("0")
            .required(false)
        )
        .arg(
            arg!(
                -a --after <NUM> "Number of lines to include after"
            )
            .value_parser(clap::value_parser!(i32))
            .default_missing_value("0")
            .required(false)
        )
        .get_matches()
}

fn get_arguments(argument_matcher: clap::ArgMatches) -> Arguments {
    let input: Box<dyn Read> = get_input(&argument_matcher);
    let output: Box<dyn Write> = get_output(&argument_matcher);
    let regexp: Vec<String> = get_regexp(&argument_matcher);
    let before_lines: i32 = get_argument_value(&argument_matcher, "before", &0);
    let after_lines: i32 = get_argument_value(&argument_matcher, "after", &0);

    Arguments {
        input,
        output,
        regexp,
        before_lines,
        after_lines,
    }
}

fn get_regexp(argument_matcher: &clap::ArgMatches) -> Vec<String> {
    argument_matcher
        .get_many::<String>("regexp")
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
        println!("{}", input_path);
        let file_result = Box::new(File::open(input_path).unwrap());
        let reader = BufReader::new(file_result);
        input = Box::new(reader);
    }
    input
}

fn get_argument_value(
    argument_matcher: &clap::ArgMatches,
    argument_name: &str,
    default: &i32,
) -> i32 {
    *argument_matcher
        .get_one::<i32>(argument_name)
        .unwrap_or(default)

}

fn parse(arguments: Arguments) {
    let mut before_buffer: CircularBuffer<String> =
        CircularBuffer::<String>::new(arguments.before_lines.clone() as usize);
    let mut after_line: i32 = 0;
    let input = arguments.input;
    let mut output = arguments.output;
    let reader = BufReader::new(input);
    let regexps = arguments
        .regexp
        .into_iter()
        .map(|r| Regex::new(&r).expect("Invalid regular expression"))
        .collect::<Vec<Regex>>();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if after_line <= 0 && is_match_any(&line, &regexps) {
                    output_before_lines(&mut before_buffer, &mut output);
                    output_line(&line, &mut output);
                    after_line = arguments.after_lines;
                } else if after_line > 0 {
                    output_line(&line, &mut output);
                    after_line -= 1;
                } else {
                    let _ = before_buffer.add(line);
                }
            }
            Err(e) => {
                std::io::stderr()
                    .write_all(format!("Error reading line: {}", e).as_bytes())
                    .unwrap();
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

fn output_before_lines(before_buffer: &mut CircularBuffer<String>, output: &mut Box<dyn Write>) {
    while before_buffer.size() > 0 {
        if let Ok(before_line) = before_buffer.remove() {
            output_line(&before_line, output);
        }
    }
}

fn output_line(line: &String, output: &mut Box<dyn Write>) {
    output.write_all(&line.as_bytes()).unwrap();
    output.write_all(b"\n").unwrap();
}
