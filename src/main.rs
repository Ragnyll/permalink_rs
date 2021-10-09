///Permalinks the given lines of a file within a git repo

extern crate exitcode;

use clap::{App, Arg};

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;
use std::process::exit;

const PATH_PARAM: &str = "PATH";
const LINE_RANGE_PARAM: &str = "LINE_RANGE";

const GITHUB_BASE_URL: &str = "heck";
const GITLAB_BASE_URL: &str = "better";

fn main() {
    let mut git_base_urls: HashMap<String, String> = HashMap::new();
    git_base_urls.insert(String::from("git@gitlab.com"), String::from(GITLAB_BASE_URL));
    git_base_urls.insert(String::from("git@github.com"), String::from(GITLAB_BASE_URL));

    let matches = App::new("git-permalink-rs")
        .version("0.1")
        .author("Ragnyll <ragnyll@gallowzhumour.dev>")
        .about("Gets a permalink to a line set within a Git repo")
        .arg(Arg::new(PATH_PARAM).required(true).takes_value(true).about("the path of the file to permalink"))
        .arg(Arg::new(LINE_RANGE_PARAM).required(true).takes_value(true).about("the line or lines to permalink. takes format startLine[-endLine]"))
        .get_matches();

    let path = Path::new(matches.value_of(PATH_PARAM).unwrap());

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

    let base_url;
    let stdout = String::from_utf8(output.stdout.to_vec()).expect("Unable to parse stdout as valid Utf8");
    let stdout = stdout.trim();

    for key in git_base_urls.keys() {
        if stdout.contains(key) {
            base_url = git_base_urls.get(key).unwrap();
            break;
        }
    }

    // TODO: parse out the repo name
    let repo_name = "getreponame"

    let mut lines: Vec<&str> = matches.value_of(LINE_RANGE_PARAM).unwrap().split("-").collect();
    lines.retain(|&e| !e.is_empty());

    match lines.len() {
        1 => (),
        2 => (),
        _ => {
            eprintln!("LINE_RANGE must be in the format of startLine[-endLine]");
            exit(exitcode::USAGE);
        },
    };
}
