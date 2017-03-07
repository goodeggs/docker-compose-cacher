extern crate glob;
extern crate sha1;
extern crate yaml_rust;
extern crate getopts;
extern crate shellexpand;
extern crate rayon;

use getopts::Options;
use std::env;
use std::fs;
use std::fs::File;
use glob::glob;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::collections::HashSet;
use shellexpand::tilde;
use self::rayon::prelude::*;

use yaml_rust::{Yaml,YamlLoader};

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("d", "", "set cache directory", "NAME");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }

    let raw_cache_path = matches.opt_str("d").unwrap_or_else(|| format!("{}", "~/.docker-compose-cacher"));
    let cache_path = tilde(&raw_cache_path);
    fs::create_dir_all(cache_path.as_ref()).unwrap();
    let cache_path = fs::canonicalize(cache_path.as_ref()).unwrap().display().to_string();


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

  images.par_iter().enumerate().for_each(|tup| {
      let (_,image) = tup;
      handle_image(&cache_path, image);
  });
  prune_images(&cache_path, images);
}

fn prune_images(cache_path: &str, images: HashSet<&str>) {
    let filenames: HashSet<String> = images.iter().map(|image| image_to_filename(&cache_path, image)).collect();
    let globber = format!("{}/*.tgz", cache_path);
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

fn image_to_filename(cache_path: &str, image: &str) -> String {
  let mut m = sha1::Sha1::new();
  m.update(image.as_bytes());
  return format!("{}/{}.tgz", cache_path, m.digest().to_string());
}

fn image_is_cached(cache_path: &str, image: &str) -> bool {
  let filename = image_to_filename(&cache_path, &image);
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

fn save_image(cache_path: &str, image: &str) {
  let filename = image_to_filename(&cache_path, &image);
  let status = Command::new("sh")
    .arg("-c")
    .arg(format!("docker save {} | gzip > {}", image, filename))
    .status()
    .unwrap();
  assert!(status.success());
}

fn load_image(cache_path: &str, image: &str) {
  let filename = image_to_filename(&cache_path, &image);
  let status = Command::new("sh")
    .arg("-c")
    .arg(format!("gunzip -c {} | docker load", filename))
    .status()
    .unwrap();
  assert!(status.success());
}

fn handle_image(cache_path: &str, image: &str) {
  if !image_is_cached(&cache_path, &image) {
    fetch_image(&image);
    save_image(&cache_path, &image);
  } else {
    load_image(&cache_path, &image);
  }
}
