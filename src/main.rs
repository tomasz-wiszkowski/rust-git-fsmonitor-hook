use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};
use watchman_client::{
    prelude::{Clock, ClockSpec, DirNameTerm, Expr, NameOnly, QueryRequestCommon},
    CanonicalPath, Connector,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let Some(token) = args.get(2) else {
        bail!("usage: {} <hook-version> <token>", args[0]);
    };
    let since = match token.as_str() {
        "" => None,
        ts if ts.starts_with('c') => Some(Clock::Spec(ClockSpec::StringClock(ts.to_string()))),
        ts => {
            let ts = ts.parse::<i64>().context("invalid fsmonitor token")?;
            Some(Clock::Spec(ClockSpec::UnixTimestamp(ts / 1_000_000_000)))
        }
    };

    let client = Connector::new().connect().await?;
    let resolved = client
        .resolve_root(CanonicalPath::canonicalize(".")?)
        .await?;

    let query_response = client
        .query::<NameOnly>(
            &resolved,
            QueryRequestCommon {
                since,
                fields: vec!["name"],
                expression: Some(Expr::Not(Box::new(Expr::DirName(DirNameTerm {
                    path: PathBuf::from(".git"),
                    depth: None,
                })))),
                ..Default::default()
            },
        )
        .await?;

    let Some(files) = query_response.files else {
        return Ok(());
    };

    match query_response.clock {
        Clock::Spec(ClockSpec::StringClock(clock)) => print!("{}\0", clock),
        Clock::Spec(ClockSpec::UnixTimestamp(_)) => bail!("watchman returned a timestamp clock"),
        Clock::ScmAware(_) => bail!("watchman returned an scm-aware clock"),
    }

    for file in files {
        let filename = file.name.into_inner();
        print!("{}\0", filename.to_str().context("non-UTF-8 filename")?);
    }

    Ok(())
}
