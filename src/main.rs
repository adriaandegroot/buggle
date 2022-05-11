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
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut vec : Vec<BuggleResult> = Vec::with_capacity(5);
    vec.push(run_query("me     ".to_string(), build_assigned_query("adridg%40FreeBSD.org")).await);
    vec.push(run_query("desktop".to_string(), build_assigned_query("desktop%40FreeBSD.org")).await);
    vec.push(run_query("kde    ".to_string(), build_assigned_query("kde%40FreeBSD.org")).await);

    for buggle in &vec {
        match buggle.count {
            None => println!("{} No results", buggle.id),
            Some(n) => println!("{} count {}", buggle.id, n),
        };
    }
    Ok(())
}
