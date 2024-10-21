use std::{collections::HashMap, path::PathBuf};

use crate::{
    object::Vertex,
    types::{Vec2, Vec3},
};

pub const SQRT_3: f32 = 1.732_050_8;

#[derive(Clone)]
pub enum FilePath {
    FileName(&'static str),
    AbsolutePath(PathBuf),
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let href = location.href().unwrap();
    let href = href.split('/').collect::<Vec<_>>();
    let href = href[..href.len() - 1].join("/");
    let base = reqwest::Url::parse(&format!("{}/res/", href)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_binary(file: FilePath) -> anyhow::Result<Vec<u8>> {
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

pub async fn load_text(file: FilePath) -> anyhow::Result<String> {
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

pub fn load_obj(data: &str) -> (Vec<Vertex>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut groups: Vec<Vec<u32>> = Vec::new();

    for line in data
        .lines()
        .filter(|e| !matches!(e.chars().next(), Some('#') | None))
    {
        let seprated = line.split(' ').collect::<Vec<_>>();
        match seprated.first() {
            Some(&"v") => {
                let x = seprated[1].parse::<f32>().unwrap();
                let y = seprated[2].parse::<f32>().unwrap();
                let z = seprated[3].parse::<f32>().unwrap();
                positions.push(Vec3::new(x, y, z));
            }
            Some(&"vn") => {
                let x = seprated[1].parse::<f32>().unwrap();
                let y = seprated[2].parse::<f32>().unwrap();
                let z = seprated[3].parse::<f32>().unwrap();
                normals.push(Vec3::new(x, y, z));
            }
            Some(&"vt") => {
                let x = seprated[1].parse::<f32>().unwrap();
                let y = seprated[2].parse::<f32>().unwrap();
                uvs.push(Vec2::new(x, y));
            }
            Some(&"f") => {
                let mut arr: Vec<Vec<u32>> = (1..4)
                    .map(|i| {
                        seprated[i]
                            .split('/')
                            .map(|e| e.parse::<u32>().unwrap().saturating_sub(1))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                groups.append(&mut arr);
            }
            _ => {}
        }
    }

    let mut indices = Vec::new();
    let mut vertices = Vec::new();
    let mut vert_normal_to_index = HashMap::new();
    for group in groups {
        let key = (group[0], group[1], group[2]);
        if let Some(e) = vert_normal_to_index.get(&key) {
            indices.push(*e)
        } else {
            let index = vertices.len() as u32;
            indices.push(index);
            vertices.push(Vertex {
                position: positions[group[0] as usize],
                uv: uvs[group[1] as usize],
                normal: normals[group[2] as usize],
            });
            vert_normal_to_index.insert(key, index);
        }
    }

    (vertices, indices)
}
