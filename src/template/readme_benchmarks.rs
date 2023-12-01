/// Module that updates the readme me with timing information.
/// The approach taken is similar to how `aoc-readme-stars` handles this.
use std::{fs, io};

use crate::Day;

static MARKER: &str = "<!--- benchmarking table --->";

#[derive(Debug)]
pub enum Error {
    Parser(String),
    IO(io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

#[derive(Clone, Debug)]
pub struct Benchmark {
    pub day: Day,
    pub part_1: Option<(String, String)>,
    pub part_2: Option<(String, String)>,
    pub total_nanos: f64,
}

pub struct TablePosition {
    pos_start: usize,
    pos_end: usize,
}

#[must_use]
pub fn get_path_for_bin(day: Day) -> String {
    format!("./src/bin/{day}.rs")
}

fn locate_table(readme: &str) -> Result<TablePosition, Error> {
    let matches: Vec<_> = readme.match_indices(MARKER).collect();

    if matches.len() > 2 {
        return Err(Error::Parser(
            "{}: too many occurences of marker in README.".into(),
        ));
    }

    let pos_start = matches
        .first()
        .map(|m| m.0)
        .ok_or_else(|| Error::Parser("Could not find table start position.".into()))?;

    let pos_end = matches
        .last()
        .map(|m| m.0 + m.1.len())
        .ok_or_else(|| Error::Parser("Could not find table end position.".into()))?;

    Ok(TablePosition { pos_start, pos_end })
}

fn construct_table(prefix: &str, benchmarks: Vec<Benchmark>, total_millis: f64) -> String {
    let header = format!("{prefix} Benchmarks");

    let mut lines: Vec<String> = vec![
        MARKER.into(),
        header,
        String::new(),
        "| Day | Part 1 | Part 2 |".into(),
        "| :---: | :---: | :---:  |".into(),
    ];

    for bench in benchmarks {
        println!("{:?}", bench);
        let path = get_path_for_bin(bench.day);
        let (p1_time, p1_bytes) = bench.part_1.unwrap_or_else(|| ("-".into(), "-".into()));
        let (p2_time, p2_bytes) = bench.part_2.unwrap_or_else(|| ("-".into(), "-".into()));

        lines.push(format!(
            "| [Day {}]({}) | `{}` `{}` | `{}` `{}` |",
            bench.day.into_inner(),
            path,
            p1_time,
            p1_bytes,
            p2_time,
            p2_bytes
        ));
    }

    lines.push(String::new());
    lines.push(format!("**Total time: {total_millis:.2}ms**\n"));
    lines.push(MARKER.into());

    lines.join("\n")
}

fn update_content(s: &mut String, timings: Vec<Benchmark>, total_millis: f64) -> Result<(), Error> {
    let positions = locate_table(s)?;
    let table = construct_table("##", timings, total_millis);
    s.replace_range(positions.pos_start..positions.pos_end, &table);
    Ok(())
}

pub fn update(timings: Vec<Benchmark>, total_millis: f64) -> Result<(), Error> {
    let path = "README.md";
    let mut readme = String::from_utf8_lossy(&fs::read(path)?).to_string();
    update_content(&mut readme, timings, total_millis)?;
    fs::write(path, &readme)?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "test_lib")]
mod tests {
    use super::{update_content, Benchmark, MARKER};
    use crate::day;

    fn get_mock_timings() -> Vec<Benchmark> {
        vec![
            Benchmark {
                day: day!(1),
                part_1: Some(("10ms".into(), "10B".into())),
                part_2: Some(("20ms".into(), "20B".into())),
                total_nanos: 3e+10,
            },
            Benchmark {
                day: day!(2),
                part_1: Some(("30ms".into(), "30B".into())),
                part_2: Some(("40ms".into(), "40B".into())),
                total_nanos: 7e+10,
            },
            Benchmark {
                day: day!(4),
                part_1: Some(("40ms".into(), "40B".into())),
                part_2: Some(("50ms".into(), "50B".into())),
                total_nanos: 9e+10,
            },
        ]
    }

    #[test]
    #[should_panic]
    fn errors_if_marker_not_present() {
        let mut s = "# readme".to_string();
        update_content(&mut s, get_mock_timings(), 190.0).unwrap();
    }

    #[test]
    #[should_panic]
    fn errors_if_too_many_markers_present() {
        let mut s = format!("{} {} {}", MARKER, MARKER, MARKER);
        update_content(&mut s, get_mock_timings(), 190.0).unwrap();
    }

    #[test]
    fn updates_empty_benchmarks() {
        let mut s = format!("foo\nbar\n{}{}\nbaz", MARKER, MARKER);
        update_content(&mut s, get_mock_timings(), 190.0).unwrap();
        assert_eq!(s.contains("## Benchmarks"), true);
    }

    #[test]
    fn updates_existing_benchmarks() {
        let mut s = format!("foo\nbar\n{}{}\nbaz", MARKER, MARKER);
        update_content(&mut s, get_mock_timings(), 190.0).unwrap();
        update_content(&mut s, get_mock_timings(), 190.0).unwrap();
        assert_eq!(s.matches(MARKER).collect::<Vec<&str>>().len(), 2);
        assert_eq!(s.matches("## Benchmarks").collect::<Vec<&str>>().len(), 1);
    }

    #[test]
    fn format_benchmarks() {
        let mut s = format!("foo\nbar\n{}\n{}\nbaz", MARKER, MARKER);
        update_content(&mut s, get_mock_timings(), 190.0).unwrap();
        let expected = [
            "foo",
            "bar",
            "<!--- benchmarking table --->",
            "## Benchmarks",
            "",
            "| Day | Part 1 | Part 2 |",
            "| :---: | :---: | :---:  |",
            "| [Day 1](./src/bin/01.rs) | `10ms` `10B` | `20ms` `20B` |",
            "| [Day 2](./src/bin/02.rs) | `30ms` `30B` | `40ms` `40B` |",
            "| [Day 4](./src/bin/04.rs) | `40ms` `40B` | `50ms` `50B` |",
            "",
            "**Total time: 190.00ms**",
            "",
            "<!--- benchmarking table --->",
            "baz",
        ]
        .join("\n");
        assert_eq!(s, expected);
    }
}
