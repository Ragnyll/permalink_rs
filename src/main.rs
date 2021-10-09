///Permalinks the given lines of a file within a git repo

extern crate exitcode;

use clap::{App, Arg};

use pathdiff;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::exit;

const PATH_PARAM: &str = "PATH";
const LINE_RANGE_PARAM: &str = "LINE_RANGE";

fn main() {
    let mut git_commit_link_scheme: HashMap<String, String> = HashMap::new();
    git_commit_link_scheme.insert(String::from("git@gitlab.com"), String::from("https://gitlab.com/REPO-NAME/-/blob/COMMIT/FILE#LINE_RANGE"));
    git_commit_link_scheme.insert(String::from("git@github.com"), String::from("https://github.com/REPO-NAME/blob/COMMIT/FILE#LINE_RANGE"));

    let matches = App::new("git-permalink-rs")
        .version("0.1")
        .author("Ragnyll <ragnyll@gallowzhumour.dev>")
        .about("Gets a permalink to a line set within a Git repo")
        .arg(Arg::new(PATH_PARAM).required(true).takes_value(true).about("the path of the file to permalink"))
        .arg(Arg::new(LINE_RANGE_PARAM).required(true).takes_value(true).about("the line or lines to permalink. takes format startLine[-endLine]"))
        .get_matches();

    let path = Path::new(matches.value_of(PATH_PARAM).unwrap());
    let path = fs::canonicalize(Path::new(path)).unwrap();

    if path.exists() {
        // get the dir of file
        let parent_dir = path.parent().expect(&format!("{} is not a valid file path", path.to_str().unwrap()));
        env::set_current_dir(parent_dir).expect(&format!("Unable to change directory to {}", parent_dir.to_str().unwrap()))
    } else {
        eprintln!("{} is not a valid file path", path.to_str().unwrap());
        exit(exitcode::USAGE)
    }

    let output = Command::new("git")
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()
        .unwrap();

    if output.status.code() != Some(0) {
        eprintln!("Unable to run git commands on {}", path.to_str().unwrap());
        exit(exitcode::USAGE)
    }

    let mut url_pattern = "";
    let stdout = String::from_utf8(output.stdout.to_vec()).expect("Unable to parse stdout as valid Utf8");
    let stdout = stdout.trim();

    for key in git_commit_link_scheme.keys() {
        if stdout.contains(key) {
            url_pattern = git_commit_link_scheme.get(key).unwrap();
            break;
        }
    }

    let repo_name = stdout.split(":").collect::<Vec::<&str>>()[1];
    let repo_name = String::from(repo_name).replace(".git", "");

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .unwrap();
    if output.status.code() != Some(0) {
        eprintln!("Unable to run git commands on {}", path.to_str().unwrap());
        exit(exitcode::USAGE)
    }
    let current_commit = String::from_utf8(output.stdout.to_vec()).expect("Unable to parse stdout as valid Utf8");
    let current_commit = current_commit.trim();

    let mut lines: Vec<&str> = matches.value_of(LINE_RANGE_PARAM).unwrap().split("-").collect();
    lines.retain(|&e| !e.is_empty());

    let line_range = match lines.len() {
        1 => {
            format!("L{}", lines[0])
        },
        2 => {
            format!("L{}-L{}", lines[0], lines[1])
        },
        _ => {
            eprintln!("LINE_RANGE must be in the format of startLine[-endLine]");
            exit(exitcode::USAGE);
        },
    };

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .unwrap();
    if output.status.code() != Some(0) {
        eprintln!("Unable to run git commands on {}", path.to_str().unwrap());
        exit(exitcode::USAGE)
    }
    let git_repo_base_path = String::from_utf8(output.stdout.to_vec()).expect("Unable to parse stdout as valid Utf8");

    let relative_path_to_file = pathdiff::diff_paths(Path::new(&path), Path::new(&git_repo_base_path)).unwrap();
    // for some reason there is always an unnessecary ../ just drop it.
    let relative_path_to_file = String::from(relative_path_to_file.into_os_string().to_str().unwrap()).replace("../", "");
    let relative_path_to_file = relative_path_to_file.split("/").collect::<Vec<&str>>().into_iter().skip(1).collect::<Vec<&str>>().join("/");

    let final_link = url_pattern.replace("REPO-NAME", &repo_name).replace("COMMIT", &current_commit).replace("FILE", &relative_path_to_file).replace("LINE_RANGE", &line_range);

    println!("{}", final_link)
}
