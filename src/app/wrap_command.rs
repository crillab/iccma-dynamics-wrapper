// iccma21-dynamics-wrapper
// Copyright (C) 2020  Artois University and CNRS
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//
// Contributors:
//   *   CRIL - initial API and implementation

use std::{
    convert::TryFrom,
    fs::File,
    io::BufRead,
    io::{BufReader, Cursor, Read, Seek, SeekFrom, Write},
};

use anyhow::{anyhow, Context, Result};
use crusti_app_helper::{AppSettings, Arg, Command, SubCommand};
use crusti_arg::{solutions, ArgumentSet};

pub(crate) struct WrapCommand;

const CMD_NAME: &str = "wrap";

const ARG_SOLVER: &str = "SOLVER";
const ARG_PROBLEM: &str = "PROBLEM";
const ARG_INPUT_FILE: &str = "INPUT_FILE";
const ARG_INPUT_FORMAT: &str = "INPUT_FORMAT";
const ARG_ARGUMENT: &str = "ARGUMENT";
const ARG_MODIFICATION_FILE: &str = "MODIFICATION_FILE";

impl WrapCommand {
    pub fn new() -> Self {
        WrapCommand
    }
}

pub enum QueryType {
    SE,
    EE,
    CE,
    DC(String),
    DS(String),
}

impl QueryType {
    fn command_arguments(&self, problem: &str, input_file: &str, file_format: &str) -> Vec<String> {
        let mut default_arguments = vec![
            "-p".to_string(),
            problem.to_string(),
            "-f".to_string(),
            input_file.to_string(),
            "-fo".to_string(),
            file_format.to_string(),
        ];
        match self {
            QueryType::SE | QueryType::EE | QueryType::CE => default_arguments,
            QueryType::DC(arg) | QueryType::DS(arg) => {
                default_arguments.push("-a".to_string());
                default_arguments.push(arg.clone());
                default_arguments
            }
        }
    }

    fn answer_reading_function(&self) -> Box<dyn Fn(&mut dyn BufRead) -> Result<String>> {
        fn compose_rw<T, R, W>(
            reading_fn: &'static R,
            writing_fn: &'static W,
        ) -> Box<dyn Fn(&mut dyn BufRead) -> Result<String>>
        where
            R: Fn(&mut dyn BufRead) -> Result<T>,
            W: Fn(&mut dyn Write, &T) -> Result<()>,
        {
            Box::new(move |reader| -> Result<String> {
                let read = reading_fn(reader).context("while reading child process stdout")?;
                let mut cursor = Cursor::new(vec![]);
                writing_fn(&mut cursor, &read)?;
                cursor.seek(SeekFrom::Start(0)).unwrap();
                let mut out = Vec::new();
                cursor.read_to_end(&mut out).unwrap();
                Ok(String::from_utf8(out).unwrap())
            })
        }
        match self {
            QueryType::SE => compose_rw(&solutions::read_extension, &solutions::write_extension),
            QueryType::EE => compose_rw(&solutions::read_extension_set, &|w, s| {
                solutions::write_extension_set(w, &s.iter().collect::<Vec<&ArgumentSet<String>>>())
            }),
            QueryType::CE => compose_rw(&solutions::read_extension_count, &|w, c| {
                solutions::write_extension_count(w, *c)
            }),
            QueryType::DC(_) | QueryType::DS(_) => {
                compose_rw(&solutions::read_acceptance_status, &|w, b| {
                    solutions::write_acceptance_status(w, *b)
                })
            }
        }
    }
}

impl TryFrom<(&str, Option<&str>)> for QueryType {
    type Error = anyhow::Error;

    fn try_from(value: (&str, Option<&str>)) -> Result<Self, Self::Error> {
        let (problem, arg) = value;
        let splits = problem.split('-').collect::<Vec<&str>>();
        let err_builder = |s| anyhow!(r#""{}" is not a valid dynamic track"#, s);
        if splits.len() != 3
            || !vec!["CO", "GR", "PR", "ST", "SST", "STG", "ID"].contains(&splits[1])
            || splits[2] != "D"
        {
            return Err(err_builder(problem));
        }
        let ok_if_no_arg = |q: QueryType| {
            if arg.is_none() {
                Ok(q)
            } else {
                Err(anyhow!(
                    r#"problem "{}" does not require an argument but one is provided"#,
                    problem
                ))
            }
        };
        let on_missing_arg = || {
            anyhow!(
                r#"problem "{}" requires an argument none is provided"#,
                problem
            )
        };
        match splits[0] {
            "SE" => ok_if_no_arg(QueryType::SE),
            "EE" => ok_if_no_arg(QueryType::EE),
            "CE" => ok_if_no_arg(QueryType::CE),
            "DC" => Ok(QueryType::DC(arg.ok_or(on_missing_arg())?.to_string())),
            "DS" => Ok(QueryType::DS(arg.ok_or(on_missing_arg())?.to_string())),
            _ => Err(err_builder(problem)),
        }
    }
}

impl<'a> Command<'a> for WrapCommand {
    fn name(&self) -> &str {
        CMD_NAME
    }

    fn clap_subcommand(&self) -> crusti_app_helper::App<'a, 'a> {
        SubCommand::with_name(CMD_NAME)
            .about(
                "wraps an argumentation solver and a dynamics file to provide on-the-fly dynamics",
            )
            .setting(AppSettings::DisableVersion)
            .arg(
                Arg::with_name(ARG_SOLVER)
                    .long("solver")
                    .short("s")
                    .takes_value(true)
                    .help("sets the solver to call")
                    .required(true),
            )
            .arg(
                Arg::with_name(ARG_PROBLEM)
                    .long("problem")
                    .short("p")
                    .takes_value(true)
                    .help("sets the problem to solve")
                    .required(true),
            )
            .arg(
                Arg::with_name(ARG_INPUT_FILE)
                    .long("input-file")
                    .short("f")
                    .takes_value(true)
                    .help("sets the input file containing the framework")
                    .required(true),
            )
            .arg(
                Arg::with_name(ARG_INPUT_FORMAT)
                    .long("input-format")
                    .short("z")
                    .takes_value(true)
                    .help("sets the input file format")
                    .required(true),
            )
            .arg(
                Arg::with_name(ARG_ARGUMENT)
                    .long("argument")
                    .short("a")
                    .takes_value(true)
                    .help("sets the argument for acceptance decision problems"),
            )
            .arg(
                Arg::with_name(ARG_MODIFICATION_FILE)
                    .long("modification")
                    .short("m")
                    .takes_value(true)
                    .help("sets the modification file containing the dynamics of the framework")
                    .required(true),
            )
    }

    fn execute(&self, arg_matches: &crusti_app_helper::ArgMatches<'_>) -> Result<()> {
        let problem = arg_matches.value_of(ARG_PROBLEM).unwrap();
        let arg = arg_matches.value_of(ARG_ARGUMENT);
        let query = QueryType::try_from((problem, arg))?;
        let mut process = std::process::Command::new(arg_matches.value_of(ARG_SOLVER).unwrap())
            .args(query.command_arguments(
                problem,
                arg_matches.value_of(ARG_INPUT_FILE).unwrap(),
                arg_matches.value_of(ARG_INPUT_FORMAT).unwrap(),
            ))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .context("while spawning child process")?;
        let mut child_stdin = process.stdin.take().unwrap();
        let mut child_stdout = BufReader::new(process.stdout.take().unwrap());
        let mut mod_br = BufReader::new(
            File::open(arg_matches.value_of(ARG_MODIFICATION_FILE).unwrap())
                .context("while opening modification file")?,
        );
        execute_dynamics(
            &mut mod_br,
            query.answer_reading_function(),
            &mut child_stdin,
            &mut child_stdout,
        )?;
        process
            .wait()
            .with_context(|| "while waiting for the end of child process")
            .map(|_| {})
    }
}

fn execute_dynamics<F: ?Sized>(
    modifications: &mut dyn BufRead,
    answer_reading_function: Box<F>,
    child_stdin: &mut dyn Write,
    child_stdout: &mut dyn BufRead,
) -> Result<()>
where
    F: Fn(&mut dyn BufRead) -> Result<String>,
{
    const CONTEXT_WRITING: &str = "while writing to child process stdin";
    for l in modifications.lines() {
        let mod_line = l.context("while reading modification file")?;
        if mod_line.is_empty() {
            break;
        }
        let read = answer_reading_function(child_stdout)?;
        print!("{}", read);
        writeln!(child_stdin, "{}", mod_line).context(CONTEXT_WRITING)?;
    }
    let read = answer_reading_function(child_stdout)?;
    print!("{}", read);
    writeln!(child_stdin).context(CONTEXT_WRITING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_dynamics_no_dyn_acceptance_status() {
        let mut modifications = BufReader::new("".as_bytes());
        let answer_reader = QueryType::DC("a".to_string()).answer_reading_function();
        let mut cursor = Cursor::new(vec![]);
        let mut child_stdout = BufReader::new("YES\n".as_bytes());
        execute_dynamics(
            &mut modifications,
            answer_reader,
            &mut cursor,
            &mut child_stdout,
        )
        .unwrap();
        let mut out = Vec::new();
        cursor.seek(SeekFrom::Start(0)).unwrap();
        cursor.read_to_end(&mut out).unwrap();
        let child_stdin = String::from_utf8(out).unwrap();
        assert_eq!("\n", child_stdin);
    }

    #[test]
    fn test_execute_dynamics_one_dyn_acceptance_status() {
        let mut modifications = BufReader::new("+arg(a).\n".as_bytes());
        let answer_reader = QueryType::DC("a".to_string()).answer_reading_function();
        let mut cursor = Cursor::new(vec![]);
        let mut child_stdout = BufReader::new("YES\nNO\n".as_bytes());
        execute_dynamics(
            &mut modifications,
            answer_reader,
            &mut cursor,
            &mut child_stdout,
        )
        .unwrap();
        let mut out = Vec::new();
        cursor.seek(SeekFrom::Start(0)).unwrap();
        cursor.read_to_end(&mut out).unwrap();
        let child_stdin = String::from_utf8(out).unwrap();
        assert_eq!("+arg(a).\n\n", child_stdin);
    }

    #[test]
    fn test_execute_dynamics_two_dyn_acceptance_statuses() {
        let mut modifications = BufReader::new("+arg(a).\n+arg(a).\n".as_bytes());
        let answer_reader = QueryType::DC("a".to_string()).answer_reading_function();
        let mut cursor = Cursor::new(vec![]);
        let mut child_stdout = BufReader::new("YES\nYES\nNO\n".as_bytes());
        execute_dynamics(
            &mut modifications,
            answer_reader,
            &mut cursor,
            &mut child_stdout,
        )
        .unwrap();
        println!("{:?}", child_stdout);
        let mut out = Vec::new();
        cursor.seek(SeekFrom::Start(0)).unwrap();
        cursor.read_to_end(&mut out).unwrap();
        let child_stdin = String::from_utf8(out).unwrap();
        assert_eq!("+arg(a).\n+arg(a).\n\n", child_stdin);
    }

    #[test]
    fn test_execute_dynamics_wrong_answer() {
        let mut modifications = BufReader::new("+arg(a).\n".as_bytes());
        let answer_reader = QueryType::DC("a".to_string()).answer_reading_function();
        let mut cursor = Cursor::new(vec![]);
        let mut child_stdout = BufReader::new("foo\n".as_bytes());
        assert!(execute_dynamics(
            &mut modifications,
            answer_reader,
            &mut cursor,
            &mut child_stdout,
        )
        .is_err());
    }
}
