use std::path::PathBuf;

pub enum FilePath<'a> {
    FileName(&'a str),
    AbsolutePath(PathBuf),
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    let base = reqwest::Url::parse(&format!("{}/res/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_binary(file: FilePath<'_>) -> anyhow::Result<Vec<u8>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            match file {
                FilePath::FileName(name) => { let url = format_url(name);
                let v = reqwest::get(url)
                    .await?
                    .bytes()
                    .await?
                    .to_vec();
                Ok(v)},
                FilePath::AbsolutePath(path) => Err(anyhow::anyhow!(format!("Can't read filepaths on the web {:?}", path))),
            }
        } else {
           use std::path::Path;
            let path = match file {
                FilePath::FileName(name) => Path::new("res").join(name),
                FilePath::AbsolutePath(path) => path,
            };

           Ok(std::fs::read(path)?)
        }
    }
}

pub async fn load_text(file: FilePath<'_>) -> anyhow::Result<String> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            match file {
                FilePath::FileName(name) => { let url = format_url(name);
                let v = reqwest::get(url)
                    .await?
                    .text()
                    .await?;
                Ok(v)},
                FilePath::AbsolutePath(path) => Err(anyhow::anyhow!(format!("Can't read filepaths on the web {:?}", path))),
            }
        } else {
           use std::path::Path;
            let path = match file {
                FilePath::FileName(name) => Path::new("res").join(name),
                FilePath::AbsolutePath(path) => path,
            };

           Ok(std::fs::read_to_string(path)?)
        }
    }
}
