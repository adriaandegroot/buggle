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
use std::io::stdout;
use std::io::Write;

fn build_query(q : String) -> hyper::Uri {
    let qs = format!("https://bugs.freebsd.org/bugzilla/buglist.cgi?{}&ctype=csv", q);
    return qs.parse().unwrap();
}

fn build_assigned_query(email : &str) -> hyper::Uri {
    return build_query(format!("email1={}&emailassigned_to1=1&emailreporter1=1&emailtype1=exact&resolution=---", email));
}

struct BuggleResult {
    id: String,
    count: Option<u32>,
}

fn run_query(uri : hyper::Uri) -> BuggleResult {
    return BuggleResult {id: "bad".to_string(), count: None};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let uri = build_assigned_query("adridg%40freebsd.org");
    let buggle = run_query(uri);
    match buggle.count {
        None => println!("{} No results", buggle.id),
        Some(n) => println!("{} count {}", buggle.id, n),
    };

    Ok(())
}
