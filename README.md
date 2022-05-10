# Buggle

> This is the Daily Buggle, which is a social media post about bug numbers
> from some bug-tracker query. In its most simple form, this does some
> Bugzilla queries and then Tweets about it.

### Rationale

I often check some FreeBSD Bugzilla queries and then Tweet about it.
That can be automated.

I don't know Rust, so I have very arbitrarily chosen it
for development of this program. The design is fairly
straightforward, with some configuration parsing, an XML HTTP request
and XML parsing, and then an HTTP request to post something.
I did a brief check and there are libraries (crates, pardon my Rust)
for at least some of that already.

### Get Involved

Since this is a learning exercise, I don't think there's much to
get involved **with**, but feel free to open a PR. I encourage PRs
that are "early stage", so we can talk about design rationale early
in the process.

You can find me on Libera.chat -- probably in `#freebsd-desktop` or `#calamares`,
which aren't exactly on-topic for Buggle, but not totally off-topic either.
