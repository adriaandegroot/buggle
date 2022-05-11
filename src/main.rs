// Buggle
//
// A bug-count social-media program.
//
// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2022 Adriaan de Groot <groot@kde.org>
//
use hyper::body::HttpBody as _;
use hyper::Client;
use hyper_tls::HttpsConnector;

// === Query Builders
//
// These functions build an HTTPs Uri for querying FreeBSD's bugzilla.
// They are intentionally simplistic, and return a query for CSV data.
//
// Use build_assigned_query() to query bugs assigned to a given email
// address; keep in mind that this match is exact, and case-sensitive.
// In particular, FreeBSD addresses are case-sensitive.
//
// Use build_product_query() to query bugs about a given product or port.
// This might be a category/port name, or just a port name, and searches
// for the given name in product, component, and short description strings
// of the bug -- e.g. PRs that mention the name in a prominent place.
fn build_query(q : String) -> hyper::Uri {
    let qs = format!("https://bugs.freebsd.org/bugzilla/buglist.cgi?{}&ctype=csv", q);
    return qs.parse().unwrap();
}

fn build_assigned_query(email : &str) -> hyper::Uri {
    return build_query(format!("email1={}&emailassigned_to1=1&emailreporter1=1&emailtype1=exact&resolution=---", email));
}

fn build_product_query(product : &str) -> hyper::Uri {
    return build_query(format!("bug_status=__open__&f0=OP&f1=OP&f2=product&f3=component&f4=alias&f5=short_desc&f7=CP&f8=CP&j1=OR&o2=substring&o3=substring&o4=substring&o5=substring&o6=substring&v2={}&v3={}&v4={}&v5={}&v6={}", product, product, product, product, product));
}

// === Query Results
//
// Do a single query and summarize the results.
//
// Queries have an associated ID (a human-readable tag describing
// the query) and a Uri that does the work. Return values include
// the ID (for reference) and an optional count. If the query
// errors out, there is no count.
//
// Function get_buggle_results() performs a number of queries
// and returns a vector with each of the results.
struct BuggleResult {
    id: String,
    count: Option<u32>,
}

async fn run_query(id : String, uri : hyper::Uri) -> BuggleResult {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut res = client.get(uri).await.ok().unwrap();
    if res.status() != 200 {
        return BuggleResult {id: id, count: None};
    }

    let mut count = 0;
    while let Some(chunk) = res.body_mut().data().await {
        let lines = chunk.ok().unwrap();
        count += lines.split(|b| b == &10u8).count();
    }
    // There is a CSV header at the top, so if we got at least one line,
    // don't count the header.
    if count > 0 { count -= 1; }
    return BuggleResult {id: id, count: Some(count.try_into().unwrap())};
}

#[tokio::main]
async fn get_buggle_results() -> Result<Vec<BuggleResult>, Box<dyn std::error::Error + Send + Sync>> {
    let mut vec : Vec<BuggleResult> = Vec::with_capacity(5);
    vec.push(run_query("me     ".to_string(), build_assigned_query("adridg%40FreeBSD.org")).await);
    vec.push(run_query("cmake  ".to_string(), build_product_query("cmake")).await);
    vec.push(run_query("desktop".to_string(), build_assigned_query("desktop%40FreeBSD.org")).await);
    vec.push(run_query("kde    ".to_string(), build_assigned_query("kde%40FreeBSD.org")).await);

    return Ok(vec);
}

// === Summary
//
// Turns the results into a simple social-media-compatible string
// that names the bug counts.
fn summarize_buggle( v : Vec<BuggleResult> ) -> String {
    let mut s : String = "Daily buggle: ".to_string();
    let mut c = v.len();
    for buggle in &v {
        let count_s = match buggle.count {
            None => "?".to_string(),
            Some(n) => format!("{}", n),
        };
        s += &count_s;
        if c > 1 {
            s += "/";
        }
        c -= 1;
    }

    if s.ends_with('/') { s.remove(1); }
    return s;
}

fn main() {
    let r = get_buggle_results().ok().unwrap();
    println!("{}", summarize_buggle(r));
}
