use hyper::Client;
use std::borrow::Borrow;
use std::fmt::Write;
use test_crs::engine::{get_variables_from_source, SourceType};
use test_crs::syntax::parse_entries;
use test_crs::{ftw, syntax, syntax::CRSEntry, CRSError};

fn main() -> Result<(), CRSError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let config_files = syntax::parse_all_conf("coreruleset/rules")?;

    for conf in config_files {
        println!("File: {}", conf.path.display());
        for entry in conf.entries {
            println!("{}", entry);
        }
    }

    let ftw_files = glob::glob("coreruleset/tests/regression/tests/*/*.yaml")
        .unwrap()
        .filter_map(|p| p.ok())
        .filter_map(|path| match ftw::File::from_path(&path) {
            Ok(file) => Some(file),
            Err(err) => {
                eprintln!("{} {}", path.display(), err);
                None
            }
        })
        .collect::<Vec<_>>();

    let ftw_stages: Vec<&ftw::Stage> = ftw_files.iter().flat_map(|file| file.stages()).collect();

    for ftw::Stage { input, .. } in ftw_stages {
        if let Ok(req) = input.request() {
            for &source in SourceType::variants() {
                for var in get_variables_from_source(&req, source) {
                    println!("{}", var);
                }
            }
            println!();
        }
    }

    // rt.block_on(async {
    //     let client = Client::new();
    //
    //     for stage in ftw_stages {
    //         let ftw::Stage { input, output } = stage;
    //         if let Ok(req) = input.request() {
    //             let mut res = client.request(req).await?;
    //             println!("Response: {}", res.status());
    //             println!("Headers: {:#?}\n", res.headers());
    //         }
    //     }
    //
    //     // // Stream the body, writing each chunk to stdout as we get it
    //     // // (instead of buffering and printing at the end).
    //     // while let Some(next) = res.data().await {
    //     //     let chunk = next?;
    //     //     io::stdout().write_all(&chunk).await?;
    //     // }
    //     Ok(())
    // })
    // .unwrap();

    Ok(())
}
