mod profile;

use bytes::Bytes;
use flate2::read::GzDecoder;
use profile::{Location, Profile};
use prost::Message;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Read},
    path::PathBuf,
};
use structopt::StructOpt;

/// pprof_blame - A tool for analyzing pprof profiles with call stack relationships
///
/// This utility processes pprof profile files and filters samples based on the
/// relationships between functions in the call stack. It uses three patterns:
///
/// - Blame: The primary function pattern to match in the stack
/// - Parent: Function pattern that must appear as an ancestor of the blame function
/// - Exclude: Function pattern that, if present, will exclude the sample
///
/// For each sample, it checks whether:
/// 1. A function matching the blame pattern exists
/// 2. A function matching the parent pattern exists (if specified)
/// 3. The blame function is called by the parent function
/// 4. No function matching the exclude pattern is present
///
/// Usage:
///   pprof_blame --file profile.pb.gz --blame "pattern" [--parent "pattern"] [--exclude "pattern"]
///
/// Output:
///   - With parent: matched samples as percentage of parent samples
///   - Without parent: matched samples as percentage of total samples
#[derive(StructOpt, Debug)]
struct Opt {
    /// Path to the profile file (.pb.gz)
    #[structopt(long)]
    file: PathBuf,

    /// Regex pattern for functions to blame
    #[structopt(long)]
    blame: String,

    /// Optional regex pattern for parent functions
    #[structopt(long)]
    parent: Option<String>,

    /// Optional regex pattern for functions to exclude
    #[structopt(long)]
    exclude: Option<String>,
}

/// Result of analyzing a profile
#[derive(Debug)]
struct AnalysisResults {
    total_samples: usize,
    total_value: i64,
    blamed_samples: usize,
    blamed_value: i64,
    excluded_samples: usize,
    excluded_value: i64,
    parent_samples: usize,
    parent_value: i64,
    blamed_frames: HashMap<String, (usize, i64)>,
    parent_frames: HashMap<String, (usize, i64)>,
    excluded_frames: HashMap<String, (usize, i64)>,
}

impl AnalysisResults {
    fn new() -> Self {
        Self {
            total_samples: 0,
            total_value: 0,
            blamed_samples: 0,
            blamed_value: 0,
            excluded_samples: 0,
            excluded_value: 0,
            parent_samples: 0,
            parent_value: 0,
            blamed_frames: HashMap::new(),
            parent_frames: HashMap::new(),
            excluded_frames: HashMap::new(),
        }
    }

    fn percentage(&self) -> f64 {
        if self.parent_samples > 0 {
            (self.blamed_value as f64 / self.parent_value as f64) * 100.0
        } else if self.total_value > 0 {
            (self.blamed_value as f64 / self.total_value as f64) * 100.0
        } else {
            0.0
        }
    }

    fn print_summary(&self, has_parent: bool) {
        let blamed_value_ms = self.blamed_value / 1_000_000;
        let parent_value_ms = self.parent_value / 1_000_000;
        let total_value_ms = self.total_value / 1_000_000;
        let excluded_value_ms = self.excluded_value / 1_000_000;

        if has_parent {
            println!(
                "{} blamed samples ({} ms) over {} parent samples ({} ms) ({:.2}%).",
                self.blamed_samples,
                blamed_value_ms,
                self.parent_samples,
                parent_value_ms,
                self.percentage()
            );
        } else {
            println!(
                "{} blamed samples ({} ms) over {} total samples ({} ms) ({:.2}%).",
                self.blamed_samples,
                blamed_value_ms,
                self.total_samples,
                total_value_ms,
                self.percentage()
            );
        }

        if self.excluded_samples > 0 {
            println!(
                "{} samples ({} ms) were excluded.",
                self.excluded_samples, excluded_value_ms
            );
        }

        if !self.blamed_frames.is_empty() {
            println!("\nBlamed Frames:");
            for (method, (count, value)) in &self.blamed_frames {
                println!("{}: {} samples, {} ms", method, count, value / 1_000_000);
            }
        }

        if has_parent && !self.parent_frames.is_empty() {
            println!("\nParent Frames:");
            for (method, (count, value)) in &self.parent_frames {
                println!("{}: {} samples, {} ms", method, count, value / 1_000_000);
            }
        }

        if !self.excluded_frames.is_empty() {
            println!("\nExcluded Frames:");
            for (method, (count, value)) in &self.excluded_frames {
                println!("{}: {} samples, {} ms", method, count, value / 1_000_000);
            }
        }
    }
}

/// A wrapper around the profile string table for safer access
struct StringTable<'a> {
    table: &'a [String],
}

impl<'a> StringTable<'a> {
    fn new(table: &'a [String]) -> Self {
        Self { table }
    }

    fn get(&self, index: i64) -> &'a str {
        if index < 0 || index as usize >= self.table.len() {
            return "<invalid_index>";
        }
        &self.table[index as usize]
    }
}

/// Loads and decodes a profile from a file
fn load_profile(path: &PathBuf) -> io::Result<Profile> {
    let file = File::open(path)?;
    let mut decoder = GzDecoder::new(BufReader::new(file));
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    Profile::decode(Bytes::from(buf)).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Extracts function names from a sample's stack trace
fn extract_stack<'a>(
    sample: &'a profile::Sample,
    location_map: &'a HashMap<u64, &'a Location>,
    function_map: &'a HashMap<u64, &'a str>,
) -> Vec<&'a str> {
    sample
        .location_id
        .iter()
        .flat_map(|loc_id| {
            location_map.get(loc_id).into_iter().flat_map(|loc| {
                loc.line
                    .iter()
                    .filter_map(|line| function_map.get(&line.function_id).copied())
            })
        })
        .collect()
}

/// Analyzes a profile with given filter patterns
fn analyze_profile(
    profile: &Profile,
    blame_re: &Regex,
    parent_re: Option<&Regex>,
    exclude_re: Option<&Regex>,
) -> AnalysisResults {
    // Create a more efficient string table accessor
    let string_table = StringTable::new(&profile.string_table);

    // Build maps for faster lookups
    let function_map: HashMap<u64, &str> = profile
        .function
        .iter()
        .map(|f| (f.id, string_table.get(f.name)))
        .collect();

    let location_map: HashMap<u64, &Location> =
        profile.location.iter().map(|l| (l.id, l)).collect();

    let mut results = AnalysisResults::new();

    // Process each sample
    for sample in &profile.sample {
        let stack = extract_stack(sample, &location_map, &function_map);

        if stack.is_empty() {
            continue;
        }

        results.total_samples += 1;
        let value = sample.value.first().copied().unwrap_or(0);
        results.total_value += value;

        // First, check for parent frame if a parent pattern is specified
        let parent_idx = parent_re.and_then(|pattern| {
            let idx = stack.iter().position(|&name| pattern.is_match(name));

            // Track all samples that match the parent pattern
            if let Some(idx) = idx {
                results.parent_samples += 1;
                results.parent_value += value;
                let method_name = stack[idx].to_string();
                let entry = results.parent_frames.entry(method_name).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += value;
            }

            idx
        });

        // Skip processing if parent pattern specified but not found
        if parent_re.is_some() && parent_idx.is_none() {
            continue;
        }

        // Determine search range for blame frame
        let search_range = if let Some(p_idx) = parent_idx {
            // Only search frames that are ancestors of parent (before parent in stack)
            &stack[..p_idx]
        } else {
            // Search entire stack when no parent specified
            &stack[..]
        };

        // Look for blame frame in the determined search range
        let blame_idx = search_range
            .iter()
            .position(|&name| blame_re.is_match(name));

        if let Some(blame_idx) = blame_idx {
            // Check for exclusions if an exclude pattern is specified
            let has_exclude = exclude_re
                .map(|pattern| {
                    // Only check frames before the blame frame
                    search_range[..blame_idx]
                        .iter()
                        .any(|&name| pattern.is_match(name))
                })
                .unwrap_or(false);

            // Count the sample appropriately
            if has_exclude {
                results.excluded_samples += 1;
                results.excluded_value += value;
                let method_name = stack[blame_idx].to_string();
                let entry = results.excluded_frames.entry(method_name).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += value;
            } else {
                results.blamed_samples += 1;
                results.blamed_value += value;
                let method_name = stack[blame_idx].to_string();
                let entry = results.blamed_frames.entry(method_name).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += value;
            }
        }
    }

    results
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let blame_re = Regex::new(&opt.blame)?;
    let parent_re = opt.parent.as_ref().map(|s| Regex::new(s).unwrap());
    let exclude_re = opt.exclude.as_ref().map(|s| Regex::new(s).unwrap());

    let profile = load_profile(&opt.file)?;

    let results = analyze_profile(&profile, &blame_re, parent_re.as_ref(), exclude_re.as_ref());

    results.print_summary(parent_re.is_some());

    Ok(())
}
