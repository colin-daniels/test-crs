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
            match entry {
                CRSEntry::SecRule {
                    actions,
                    inputs,
                    test,
                } => {
                    println!("{:?}", test);
                    for action in actions {
                        println!("{:?}", action);
                    }
                    for input in inputs {
                        println!("{:?}", input);
                    }
                }
                CRSEntry::SecAction(actions) => {
                    for action in actions {
                        println!("{:?}", action);
                    }
                }
                _ => (),
            }
        }
    }

    let mut files = vec![];
    for entry in glob::glob("coreruleset/tests/regression/tests/*/*.yaml").unwrap() {
        if let Ok(path) = entry {
            let testfile = match ftw::File::from_path(&path) {
                Ok(testfile) => testfile,
                Err(err) => {
                    eprintln!("{} {:?}", path.display(), err);
                    Err(err)?
                }
            };
            files.push(testfile);

            let testfile = files.last().unwrap();
            for input in testfile.inputs() {
                match input.request() {
                    Ok(_) => {}
                    Err(e) => {
                        println!("err: {:?}", e);
                        println!("input: {:?}", input);
                    }
                }
            }
        }
    }

    rt.block_on(async {
        let url = "http://google.com/?q=hello";
        let url = url.parse::<hyper::Uri>().unwrap();
        if url.scheme_str() != Some("http") {
            println!("This example only works with 'http' URLs.");
            return Ok(());
        }
        test_crs::fetch_url(url).await
    })
    .unwrap();

    Ok(())
}
