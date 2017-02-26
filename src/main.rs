extern crate glob;
extern crate sha1;
extern crate yaml_rust;

use std::fs;
use std::fs::File;
use glob::glob;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::collections::HashSet;

use yaml_rust::{Yaml,YamlLoader};

static CACHE_PATH: &'static str = "/Users/bob/.docker-compose-cacher";

fn main() {
  fs::create_dir_all(CACHE_PATH).unwrap();

  let mut f = File::open("docker-compose.yml").unwrap();
  let mut s = String::new();
  f.read_to_string(&mut s).unwrap();

  let docs = YamlLoader::load_from_str(&s).unwrap();
  let doc = &docs[0];
  let services = doc["services"].as_hash().unwrap();
  let mut images = HashSet::new();
  for (_, service) in services {
      if let Yaml::String(ref image) = service["image"] {
        images.insert(image.as_str());
      }
  }

  for image in &images {
      handle_image(image);
  }
  prune_images(images);
}

fn prune_images(images: HashSet<&str>) {
    let filenames: HashSet<String> = images.iter().map(|image| image_to_filename(image)).collect();
    let globber = format!("{}/*.tgz", CACHE_PATH);
    for entry in glob(&globber).unwrap() {
        if let Ok(path) = entry {
            let f = path.display().to_string();
            if !filenames.contains(&f) {
                println!("pruning {}", path.display());
                fs::remove_file(f).unwrap();
            }
        }
    }
}

fn image_to_filename(image: &str) -> String {
  let mut m = sha1::Sha1::new();
  m.update(image.as_bytes());
  return format!("{}/{}.tgz", CACHE_PATH, m.digest().to_string());
}

fn image_is_cached(image: &str) -> bool {
  let filename = image_to_filename(&image);
  return Path::new(&filename).exists()
}

fn fetch_image(image: &str) {
  let status = Command::new("docker")
    .arg("pull")
    .arg(image)
    .status()
    .unwrap();
  assert!(status.success());
}

fn save_image(image: &str) {
  let filename = image_to_filename(&image);
  let status = Command::new("sh")
    .arg("-c")
    .arg(format!("docker save {} | gzip > {}", image, filename))
    .status()
    .unwrap();
  assert!(status.success());
}

fn load_image(image: &str) {
  let filename = image_to_filename(&image);
  let status = Command::new("sh")
    .arg("-c")
    .arg(format!("gunzip -c {} | docker load", filename))
    .status()
    .unwrap();
  assert!(status.success());
}

fn handle_image(image: &str) {
  if !image_is_cached(&image) {
    fetch_image(&image);
    save_image(&image);
  } else {
    load_image(&image);
  }
}
