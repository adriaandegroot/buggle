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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let uri = "https://bugs.freebsd.org/bugzilla/buglist.cgi?email1=adridg%40freebsd.org&emailassigned_to1=1&emailreporter1=1&emailtype1=exact&resolution=---&ctype=csv".parse()?;
    let mut res = client.get(uri).await?;

    println!("Hello, world! {}", res.status());
    while let Some(chunk) = res.body_mut().data().await {
        stdout().write_all(&chunk?)?;
    }
    Ok(())
}
