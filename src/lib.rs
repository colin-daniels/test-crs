use crate::syntax::CRSEntry;

pub mod expr;
pub mod ftw;
pub mod rules;
pub mod syntax;

use hyper::{body::HttpBody as _, Client};
use thiserror::Error;
use tokio::io::{self, AsyncWriteExt as _};

#[derive(Error, Debug)]
pub enum CRSError {
    #[error(transparent)]
    ParseError(#[from] syntax::CRSParseError),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    HyperError(#[from] hyper::Error),
    #[error(transparent)]
    FTWError(#[from] ftw::Error),
}

pub async fn fetch_url(url: hyper::Uri) -> Result<(), CRSError> {
    let client = Client::new();

    let mut res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = res.data().await {
        let chunk = next?;
        io::stdout().write_all(&chunk).await?;
    }

    println!("\n\nDone!");

    Ok(())
}
