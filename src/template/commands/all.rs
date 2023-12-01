use std::io;

use crate::template::{
    readme_benchmarks::{self, Benchmark},
    ANSI_BOLD, ANSI_ITALIC, ANSI_RESET,
};
use crate::{all_days, Day};

pub fn handle(is_release: bool, is_timed: bool) {
    let mut benchmarks: Vec<Benchmark> = vec![];

    all_days().for_each(|day| {
        if day > 1 {
            println!();
        }

        println!("{ANSI_BOLD}Day {day}{ANSI_RESET}");
        println!("------");

        let output = child_commands::run_solution(day, is_timed, is_release).unwrap();

        if output.is_empty() {
            println!("Not solved.");
        } else {
            let val = child_commands::parse_exec_bench(&output, day);
            benchmarks.push(val);
        }
    });

    if is_timed {
        let total_millis = benchmarks.iter().map(|x| x.total_nanos).sum::<f64>() / 1_000_000_f64;

        println!("\n{ANSI_BOLD}Total:{ANSI_RESET} {ANSI_ITALIC}{total_millis:.2}ms{ANSI_RESET}");
    }

    if is_release {
        println!();

        let total_millis = benchmarks.iter().map(|x| x.total_nanos).sum::<f64>() / 1_000_000_f64;
        match readme_benchmarks::update(benchmarks, total_millis) {
            Ok(()) => {
                println!("{ANSI_ITALIC}Successfully updated README with benchmarks.{ANSI_RESET}")
            }
            Err(_) => {
                eprintln!("Failed to update readme with benchmarks.");
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    BrokenPipe,
    Parser(String),
    IO(io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

#[must_use]
pub fn get_path_for_bin(day: Day) -> String {
    format!("./src/bin/{day}.rs")
}

/// All solutions live in isolated binaries.
/// This module encapsulates interaction with these binaries, both invoking them as well as parsing the timing output.
mod child_commands {
    use super::{get_path_for_bin, Error};
    use crate::Day;
    use std::{
        io::{BufRead, BufReader},
        path::Path,
        process::{Command, Stdio},
        thread,
    };

    /// Run the solution bin for a given day
    pub fn run_solution(day: Day, is_timed: bool, is_release: bool) -> Result<Vec<String>, Error> {
        // skip command invocation for days that have not been scaffolded yet.
        if !Path::new(&get_path_for_bin(day)).exists() {
            return Ok(vec![]);
        }

        let day_padded = day.to_string();
        let mut args = vec!["run", "--quiet", "--bin", &day_padded];

        if is_release {
            args.push("--release");
        }

        if is_timed {
            // mirror `--time` flag to child invocations.
            args.push("--");
            args.push("--time");
        }

        // spawn child command with piped stdout/stderr.
        // forward output to stdout/stderr while grabbing stdout lines.

        let mut cmd = Command::new("cargo")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = BufReader::new(cmd.stdout.take().ok_or(super::Error::BrokenPipe)?);
        let stderr = BufReader::new(cmd.stderr.take().ok_or(super::Error::BrokenPipe)?);

        let mut output = vec![];

        let thread = thread::spawn(move || {
            stderr.lines().for_each(|line| {
                eprintln!("{}", line.unwrap());
            });
        });

        for line in stdout.lines() {
            let line = line.unwrap();
            println!("{line}");
            output.push(line);
        }

        thread.join().unwrap();
        cmd.wait()?;

        Ok(output)
    }

    pub fn parse_exec_bench(output: &[String], day: Day) -> super::Benchmark {
        let mut bench = super::Benchmark {
            day,
            part_1: None,
            part_2: None,
            total_nanos: 0_f64,
        };

        output
            .iter()
            .filter_map(|l| {
                let part = l.split(':').next()?;
                let Some(heap_allocation) = parse_heap_allocation(l) else {
                    eprintln!("Could not parse heap allocation from line: {l}");
                    return None;
                };

                if !l.contains(" samples)") {
                    return Some((part, "", 0_f64, heap_allocation));
                }

                let Some((timing_str, nanos)) = parse_time(l) else {
                    eprintln!("Could not parse timings from line: {l}");
                    return None;
                };

                Some((part, timing_str, nanos, heap_allocation))
            })
            .for_each(|(part, timing_str, nanos, heap_allocation)| {
                if part.contains("Part 1") {
                    bench.part_1 = Some((timing_str.into(), heap_allocation.into()));
                } else if part.contains("Part 2") {
                    bench.part_2 = Some((timing_str.into(), heap_allocation.into()));
                }

                bench.total_nanos += nanos;
            });

        bench
    }

    fn parse_to_float(s: &str, postfix: &str) -> Option<f64> {
        s.split(postfix).next()?.parse().ok()
    }

    fn parse_time(line: &str) -> Option<(&str, f64)> {
        // for possible time formats, see: https://github.com/rust-lang/rust/blob/1.64.0/library/core/src/time.rs#L1176-L1200
        let str_timing = line
            .split(" samples)")
            .next()?
            .split('(')
            .last()?
            .split('@')
            .next()?
            .trim();

        let parsed_timing = match str_timing {
            s if s.contains("ns") => s.split("ns").next()?.parse::<f64>().ok(),
            s if s.contains("µs") => parse_to_float(s, "µs").map(|x| x * 1000_f64),
            s if s.contains("ms") => parse_to_float(s, "ms").map(|x| x * 1_000_000_f64),
            s => parse_to_float(s, "s").map(|x| x * 1_000_000_000_f64),
        }?;

        Some((str_timing, parsed_timing))
    }

    fn parse_heap_allocation(line: &str) -> Option<&str> {
        let str_heap_allocation = line.split(") (").last()?.split(')').next()?.trim();

        str_heap_allocation.find('B')?;

        Some(str_heap_allocation)
    }

    /// copied from: https://github.com/rust-lang/rust/blob/1.64.0/library/std/src/macros.rs#L328-L333
    #[cfg(test)]
    #[cfg(feature = "test_lib")]
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {{
            let (a, b) = (&$a, &$b);
            assert!(
                (*a - *b).abs() < 1.0e-6,
                "{} is not approximately equal to {}",
                *a,
                *b
            );
        }};
    }

    #[cfg(test)]
    #[cfg(feature = "test_lib")]
    mod tests {
        use super::parse_exec_bench;

        use crate::day;

        #[test]
        fn test_well_formed() {
            let res = parse_exec_bench(
                &[
                    "Part 1: 0 (74.13ns @ 100000 samples) (10KB)".into(),
                    "Part 2: 10 (74.13ms @ 99999 samples) (10KB)".into(),
                    "".into(),
                ],
                day!(1),
            );
            assert_approx_eq!(res.total_nanos, 74130074.13_f64);
            assert_eq!(res.part_1.unwrap(), ("74.13ns".into(), "10KB".into()));
            assert_eq!(res.part_2.unwrap(), ("74.13ms".into(), "10KB".into()));
        }

        #[test]
        fn test_patterns_in_input() {
            let res = parse_exec_bench(
                &[
                    "Part 1: @ @ @ ( ) ms (2s @ 5 samples) (10B)".into(),
                    "Part 2: 10s (100ms @ 1 samples) (10B)".into(),
                    "".into(),
                ],
                day!(1),
            );
            assert_approx_eq!(res.total_nanos, 2100000000_f64);
            assert_eq!(res.part_1.unwrap(), ("2s".into(), "10B".into()));
            assert_eq!(res.part_2.unwrap(), ("100ms".into(), "10B".into()));
        }

        #[test]
        fn test_missing_parts() {
            let res = parse_exec_bench(
                &[
                    "Part 1: ✖        ".into(),
                    "Part 2: ✖        ".into(),
                    "".into(),
                ],
                day!(1),
            );
            assert_approx_eq!(res.total_nanos, 0_f64);
            assert_eq!(res.part_1.is_none(), true);
            assert_eq!(res.part_2.is_none(), true);
        }
    }
}
