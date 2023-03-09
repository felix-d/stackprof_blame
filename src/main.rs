use std::{path::PathBuf, fs, collections::{HashMap}, rc::Rc};
use regex::Regex;

use clap::Parser;
use serde_json::{from_str, Value};

#[derive(Parser, Debug)]
#[command(author = "Felix Descoteaux", version, about, long_about = None)]
struct Cli {
    /// The path of the Stackprof profile.
    profile_path: PathBuf,

    /// The matcher for which to extract data.
    #[arg(short, long, value_name = "MATCHER")]
    blame: String,

    /// The matcher to exclude
    #[arg(short, long, value_name = "MATCHER")]
    exclude: String,
}

#[derive(Debug)]
struct Sample {
    stack: Vec<Rc<Frame>>,
    weight: u64,
    duration: u64,
    blamed: bool,
}

impl Sample {
    fn new() -> Self {
        Self {
            stack: vec![],
            weight: 0,
            duration: 0,
            blamed: false,
        }
    }
}

#[derive(Debug)]
struct BlameResult<'a> {
    profile: &'a Profile,
    samples: Vec<Sample>
}

impl<'a> BlameResult<'a> {
    fn new(profile: &'a Profile) -> Self {
        Self {
            profile,
            samples: vec![]
        }
    }

    fn print_report(&self) {
        let total_blamed_duration: u64 =
            self.samples
                .iter()
                .filter(|v| v.blamed)
                .map(|v| v.duration)
                .sum();

        println!(
            "{}ms spent in blamed samples over {}ms ({:.1}%)",
            total_blamed_duration / 1000,
            self.profile.total_duration / 1000,
            total_blamed_duration as f64 / self.profile.total_duration as f64 * 100f64,
        )
    }
}

#[derive(Debug)]
struct Frame {
    name: String,
    file: String,
}

impl Frame {
    fn from_json(json: Value) -> Frame {
        let name = json["name"].as_str().unwrap();
        let file = json["file"].as_str().unwrap();

        Frame {
          name: name.to_string(),
          file: file.to_string(),
        }
    }

    fn matches(&self, matcher: &Regex) -> bool {
        matcher.is_match(&self.name) || matcher.is_match(&self.file)
    }
}

#[derive(Debug)]
struct Profile {
    blame_matcher: Regex,
    exclude_matcher: Regex,
    frames: HashMap<String, Rc<Frame>>,
    raw: Vec<String>,
    raw_timestamp_deltas: Vec<u64>,
    total_duration: u64,
    total_weight: u64
}

impl Profile {
    fn new(json: Value, blame_matcher: Regex, exclude_matcher: Regex) -> Self {
        let frames =
            json["frames"]
                .as_object()
                .expect("The profile is not valid.")
                .clone()
                .into_iter()
                .map(|(k, v)| (k, Rc::new(Frame::from_json(v))))
                .collect();

        let raw =
            json["raw"]
                .as_array()
                .expect("The profile is not valid.")
                .clone()
                .into_iter()
                .map(|v| v.as_u64().expect("The profile is not valid.").to_string())
                .collect();

        let raw_timestamp_deltas: Vec<u64> =
            json["raw_timestamp_deltas"]
                .as_array()
                .expect("The profile is not valid.")
                .clone()
                .into_iter()
                .map(|v| v.as_u64().expect("The profile is not valid."))
                .collect();

        let total_duration = raw_timestamp_deltas.iter().sum();

        let total_weight =
            json["samples"]
                .as_u64()
                .expect("The profile is not valid.");

        Self {
            frames,
            blame_matcher,
            exclude_matcher,
            raw,
            raw_timestamp_deltas,
            total_duration,
            total_weight
        }
    }

    fn blame(&self) -> BlameResult {
        let mut result = BlameResult::new(self);
        let mut i = 0;
        let mut d: usize = 0;

        while i < self.raw.len() {
            let mut sample = Sample::new();
            let stack_height = self.raw[i].parse::<usize>().unwrap();
            i += 1;
            for _ in 0..stack_height {
                let frame_id: &String = &self.raw[i];
                let frame = self.frames.get(frame_id).unwrap().clone();
                sample.stack.push(frame);
                i += 1;
            }
            let weight = self.raw[i].parse::<u64>().unwrap();
            let duration: u64 = self.raw_timestamp_deltas[d..d + weight as usize].iter().sum();
            sample.duration = duration;
            d += weight as usize;
            sample.weight = weight;

            let mut blamed = false;
            for frame in &sample.stack {
                if blamed && frame.matches(&self.exclude_matcher) {
                    blamed = false;
                    break;
                }
                if frame.matches(&self.blame_matcher) {
                    blamed = true;
                }
            }

            sample.blamed = blamed;

            result.samples.push(sample);

            i += 1;
        }

        result
    }
}

fn main() {
    let cli = Cli::parse();

    let profile_data = fs::read_to_string(cli.profile_path)
      .expect("The file does not exist.");

    let json: Value = from_str(&profile_data)
      .expect("The profile is not valid json.");

    let blame_matcher = Regex::new(&cli.blame)
        .expect("The blame matcher is not a valid regular expression.");

    let exclude_matcher = Regex::new(&cli.exclude)
        .expect("The exclude matcher is not a valid regular expression.");

    let mut profile = Profile::new(
        json,
        blame_matcher,
        exclude_matcher,
    );

    let result = profile.blame();
    result.print_report();
}
