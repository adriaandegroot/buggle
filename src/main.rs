// Buggle
//
// A bug-count social-media program.
//
// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2022 Adriaan de Groot <groot@kde.org>
//
use config::Config;
use hyper::body::HttpBody as _;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::Serialize;
use std::env;

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
fn build_query(q: String) -> hyper::Uri {
    let qs = format!(
        "https://bugs.freebsd.org/bugzilla/buglist.cgi?{}&ctype=csv",
        q
    );
    return qs.parse().unwrap();
}

fn build_assigned_query(email: String) -> hyper::Uri {
    return build_query(format!(
        "email1={}&emailassigned_to1=1&emailreporter1=1&emailtype1=substring&resolution=---",
        email
    ));
}

fn build_product_query(product: String) -> hyper::Uri {
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

struct BuggleFlags {
    verbose: bool,
    dry_run: bool,
    twitter: bool,
}

async fn run_query(id: String, uri: hyper::Uri, flags: &BuggleFlags) -> BuggleResult {
    if flags.verbose {
        println!("Query for {} = {}", id, uri.to_string());
    }
    if flags.dry_run {
        return BuggleResult {
            id: id,
            count: None,
        };
    }

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut res = client.get(uri).await.ok().unwrap();
    if res.status() != 200 {
        return BuggleResult {
            id: id,
            count: None,
        };
    }

    let mut count = 0;
    while let Some(chunk) = res.body_mut().data().await {
        let lines = chunk.ok().unwrap();
        count += lines.split(|b| b == &10u8).count();
    }
    // There is a CSV header at the top, so if we got at least one line,
    // don't count the header.
    if count > 0 {
        count -= 1;
    }
    return BuggleResult {
        id: id,
        count: Some(count.try_into().unwrap()),
    };
}

#[tokio::main]
async fn get_buggle_results(
    queries: Vec<config::Value>,
    flags: &BuggleFlags,
) -> Result<Vec<BuggleResult>, Box<dyn std::error::Error + Send + Sync>> {
    let mut vec: Vec<BuggleResult> = Vec::with_capacity(queries.len());

    for q in queries {
        let kv = q.into_table().unwrap();
        let kind = kv.get::<String>(&"kind".to_string()).unwrap().to_string();
        let smatch = kv.get::<String>(&"match".to_string()).unwrap().to_string();
        let id = kv.get::<String>(&"name".to_string()).unwrap().to_string();

        if kind == "owner".to_string() {
            vec.push(run_query(id, build_assigned_query(smatch), &flags).await);
        } else if kind == "product" {
            vec.push(run_query(id, build_product_query(smatch), &flags).await);
        } else {
            println!("Unknown query name={} kind={}\n", id, kind);
        }
    }

    return Ok(vec);
}

// === Social Media things

#[derive(Serialize)]
struct TwitterStatus {
    text: String,
}

#[tokio::main]
async fn send_twit_impl(
    text: String,
    token: &oauth::Token,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let uri = "https://api.twitter.com/2/tweets";
    // There are no POST parameters, send a unit; all the data is in JSON.
    let header = oauth::post(uri, &(), &token, oauth::HmacSha1);
    let body_data = TwitterStatus { text: text };

    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .uri(uri)
        .header("content-type", "application/json")
        .header(hyper::header::AUTHORIZATION, header)
        .body(hyper::Body::from(
            serde_json::to_string(&body_data).unwrap(),
        ))?;

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let resp = client.request(req).await?;

    println!("Response: {}", resp.status());
    return Ok(());
}

fn send_twit(text: String, token: &oauth::Token) {
    send_twit_impl(text, token).ok();
}

// === Summary
//
// Turns the results into a simple social-media-compatible string
// that names the bug counts.
fn summarize_buggle(v: Vec<BuggleResult>) -> String {
    let mut s: String = "Daily buggle: ".to_string();
    let mut c = v.len();
    for buggle in &v {
        let count_s = match buggle.count {
            None => "?".to_string(),
            Some(n) => format!("{}", n),
        };
        if s.ends_with('/') {
            s += " ";
        }
        s += &count_s;
        s += " (";
        s += &buggle.id;
        s += ")";
        if c > 1 {
            s += " /";
        }
        c -= 1;
    }

    if s.ends_with('/') {
        s.remove(1);
    }
    return s;
}

// === Option Handling
//

fn main() {
    let settings = Config::builder()
        .add_source(config::File::with_name("buggle"))
        .add_source(config::File::with_name("buggle-auth").required(false))
        .build()
        .unwrap();

    let args: Vec<String> = env::args().collect();
    let flags = BuggleFlags {
        verbose: args.contains(&"-V".to_string()) || args.contains(&"--verbose".to_string()),
        dry_run: args.contains(&"-n".to_string()) || args.contains(&"--dry-run".to_string()),
        twitter: args.contains(&"--twitter".to_string()),
    };

    let queries = settings.get_array("queries").unwrap();
    let r = get_buggle_results(queries, &flags).ok().unwrap();
    let summary = summarize_buggle(r);
    println!("{}", summary);

    if flags.twitter {
        let token = oauth::Token::from_parts(
            settings.get_string("twitter-app.key").ok().unwrap(),
            settings.get_string("twitter-app.secret").ok().unwrap(),
            settings.get_string("twitter-user.key").ok().unwrap(),
            settings.get_string("twitter-user.secret").ok().unwrap(),
        );
        send_twit(summary, &token);
    }
}
