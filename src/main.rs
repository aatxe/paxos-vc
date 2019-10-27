#[macro_use]
extern crate clap;
#[macro_use]
extern crate fehler;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

use clap::{Arg, App};

#[tokio::main]
async fn main() -> Result<(), fehler::Exception> {
    let cli = App::new("paxos-vc")
        .version("1.0")
        .author("Aaron Weiss <awe@pdgn.co>")
        .about("view change in paxos")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("HOSTNAME")
                .help("Sets the hostname of the current process")
                .takes_value(true)
                .required(true)
        ).arg(
            Arg::with_name("hostfile")
                .short("h")
                .long("hosts")
                .value_name("HOSTFILE")
                .help("Sets the configuration for all hosts in the system")
        ).arg(
            Arg::with_name("testcase")
                .short("t")
                .long("test")
                .value_name("TEST_CASE")
                .help("Sets which test case to run, based on assignment description")
                .takes_value(true)
        );
    let matches = cli.get_matches();
    let hostname = matches.value_of("name").unwrap();
    let hostfile_path = matches.value_of("hostfile").unwrap_or("hosts");
    let test_case = value_t!(matches, "testcase", TestCase).unwrap_or_default();

    let hostfile = load_hostfile(hostfile_path)?;

    unimplemented!()
}

#[throws(io::Error)]
fn load_hostfile<P: AsRef<Path>>(path: P) -> Vec<String> {
    let mut buffer = String::new();
    File::open(path)?.read_to_string(&mut buffer)?;
    buffer.lines().map(|s| s.to_owned()).collect()
}

#[repr(u8)]
pub enum TestCase {
    /// Basic change: Start with container 0 as leader (view 0) and successfully change to container
    /// 1 (view 1) when the timeout was triggered. There is no leader crash in this scenario.
    NormalCase = 1,

    /// Full rotation: Start with container 0 and successfully do a full rotation of all containers
    /// as leaders till coming back to container 0 as leader.  That is, first view change switch to
    /// container 1, second to container 2 and so on.  There is no leader crash in this scenario.
    FullRotation = 2,

    /// New leader crashes before installing new view: Start with container 0 as leader and when
    /// timeout triggered start view change to switch to container 1. After receiving enough
    /// ViewChange messages, container 1 exits and does not complete the protocol. A correct
    /// implementation should trigger another view change to container 2 and finish the protocol.
    SingleCrash = 3,

    /// Two cascading failures of new leaders (container 1 and container 2): As above but
    /// container 2 also crashes in the middle of the protocol after receiving enough View Changes.
    /// Correct implementation should finish the view change and end up with container 3 as leader.
    TwoCrashes = 4,

    /// Three cascading failures of new leaders (containers 1, 2, and 3): As above but now there are
    /// three failed view changes because containers 1, 2, 3 exit before finishing the protocol.
    /// Correct implementation should block because there are more than 2 failures.
    ThreeCrashes = 5,
}

impl Default for TestCase {
    fn default() -> TestCase {
        TestCase::NormalCase
    }
}

impl FromStr for TestCase {
    type Err = fehler::Exception;

    #[throws]
    fn from_str(s: &str) -> TestCase  {
        match s.parse()? {
            1u8 => TestCase::NormalCase,
            2 => TestCase::FullRotation,
            3 => TestCase::SingleCrash,
            4 => TestCase::TwoCrashes,
            5 => TestCase::ThreeCrashes,
            _ => unreachable!(),
        }
    }
}
