extern crate sha1;
extern crate yaml_rust;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

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
  for (_, service) in services {
    match service["image"] {
			Yaml::String(ref image) => handle_image(image.as_str()),
      _ => continue
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
