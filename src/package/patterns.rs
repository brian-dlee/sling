// For information on these regular expressions, see https://peps.python.org/pep-0440/

// A python package name, alpha-numeric with - and _
const PACKAGE_NAME_PATTERN: &str = r"(?P<name>[-_a-z0-9]+)";
// The supported operators when specifying dependency versions: ===, ==, ~=, !=, <, <=, >, >=
const OP_PATTERN: &str = r"(?P<op>={2,3}|[<>~!]=|[<>])";
// N, N.N, N.N.N, ...
const VERSION_PATTERN: &str = r"v?(?P<version>\d+(?:\.\d+)*)";
// pre, .alpha1, -b10
const PRE_VERSION_PATTERN: &str =
    r"(?<pre>(:?[.-_])?(:?a(lpha)?|b(eta)?|c|pre|preview|rc)(?P<pre_num>\d*))?";
// post, .rev1, -r10
const POST_VERSION_PATTERN: &str = r"(?<post>(:?[.-_])?(:?r(ev)?|post)(?P<post_num>\d*))?";
// dev, .dev0, -dev10
const DEV_VERSION_PATTERN: &str = r"(?<dev>(:?[.-_])?(:?dev)(?<dev_num>\d*))?";
// +ubuntu-1, +linux.0.10.compat, +10_f
const LOCAL_VERSION_PATTERN: &str = r"((:?\+)(?P<local>[a-z0-9]+(?:[.-_][a-z0-9]+)*)";
// 0.6.1, v2022.01.01.dev, 90.1-rc3+ubuntu.01
const COMMON_VERSION_PATTERN: &str = (String::new()
    + VERSION_PATTERN
    + PRE_VERSION_PATTERN
    + POST_VERSION_PATTERN
    + DEV_VERSION_PATTERN
    + LOCAL_VERSION_PATTERN)
    .as_str();

// 1_ is used instead of 1! in some filenames
const FILENAME_EPOCH_PATTERN: &str = r"(:?(?P<epoch>\d+)[!_])?";
// Known file extensions for package files
const FILENAME_EXT_PATTERN: &str = r"\.(tar\.gz|tgz|zip|whl)";
// 1_0.6.1, 0_v2022.01.01.dev, 90.1-rc3+ubuntu.01
const FILENAME_VERSION_PATTERN: &str =
    (String::from(FILENAME_EPOCH_PATTERN) + COMMON_VERSION_PATTERN).as_str();

// 0!, 01! 1000!
const STANDARD_EPOCH_PATTERN: &str = r"(:?(?P<epoch>\d+)!)?";
// 1!0.6.1, 0!v2022.01.01.dev, 90.1-rc3+ubuntu.01
const STANDARD_VERSION_PATTERN: &str =
    (String::from(STANDARD_EPOCH_PATTERN) + COMMON_VERSION_PATTERN).as_str();

// ~=0.6.5, >0!v2022.01.01.dev,<v2023
const SINGLE_SPECIFIER_PATTERN: &str =
    (String::from(r"(:?") + OP_PATTERN + STANDARD_VERSION_PATTERN + ")").as_str();

pub(crate) static STANDARD_PACKAGE_RE: regex::Regex = build_insensitive_regex(
    (String::from("^")
        + PACKAGE_NAME_PATTERN
        + "@"
        + OP_PATTERN
        + "?"
        + STANDARD_VERSION_PATTERN
        + "$")
        .as_str(),
);
pub(crate) static FILENAME_PACKAGE_RE: regex::Regex = build_insensitive_regex(
    (String::from("^")
        + PACKAGE_NAME_PATTERN
        + "-"
        + FILENAME_VERSION_PATTERN
        + FILENAME_EXT_PATTERN
        + "$")
        .as_str(),
);

pub(crate) static SPECIFIER_RE: regex::Regex =
    build_insensitive_regex((String::from("^") + SINGLE_SPECIFIER_PATTERN + "+$").as_str());

fn build_insensitive_regex(s: &str) -> regex::Regex {
    regex::RegexBuilder::new(s)
        .case_insensitive(true)
        .build()
        .unwrap()
}
