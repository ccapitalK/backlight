extern crate argparse;

use std::fs::{read_dir, File};
use std::fmt;
use std::ffi::OsString;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::process::exit;
use std::collections::HashMap;

use argparse::{ArgumentParser, StoreTrue, StoreOption};

fn read_file_to_end(file_name: &PathBuf) -> Result<String, Error> {
    let mut ret_string = String::new();
    try!(try!(File::open(file_name.as_os_str())).read_to_string(&mut ret_string));
    Ok(ret_string)
}

fn read_file_to_usize(file_name: &PathBuf) -> Result<usize, Error> {
    let fc = try!(read_file_to_end(file_name));
    let f = fc.split_whitespace().next();
    match f {
        Some(v) => match v.parse::<usize>() {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::new(ErrorKind::Other, "Couldn't parse value"))
        },
        None => Err(Error::new(ErrorKind::Other, "Empty file"))
    }
}

fn write_file(file_name: &PathBuf, data: &[u8]) -> Result<usize, Error> {
    try!(File::create(file_name.as_os_str())).write(data)
}

struct Backlights {
    by_name: HashMap<OsString, Backlight>,
}

impl Backlights {
    pub fn new() -> Self {
        let mut hm = HashMap::new();
        for name in read_dir("/sys/class/backlight").expect("Could not open /sys/class/backlights") {
            match name {
                Ok(v) => {
                    let file_name = v.file_name();
                    let bl = Backlight::from_name(&file_name).expect("Failed to open backlight");
                    hm.insert(file_name, bl);
                }
                Err(e) => {
                    panic!("{}", e);
                },
            }
        }
        Backlights {
            by_name: hm,
        }
    }
    pub fn print_backlights(&self){
        println!("Printing device names:");
        for name in self.by_name.keys() {
            println!("{:?}", name);
        }
    }
    pub fn default_backlight(mut self) -> Result<Backlight, Error> {
        let os_default = OsString::from("default");
        Ok(if self.by_name.contains_key(&os_default) {
            self.by_name.remove(&os_default).unwrap()
        } else if self.by_name.len() > 0 {
            let key = self.by_name.keys().nth(0).unwrap().clone();
            self.by_name.remove(&key).unwrap()
        } else {
            return Err(Error::new(ErrorKind::Other, "No backlights detected"))
        })
    }
}

#[derive(Debug)]
struct Backlight {
    name: OsString,
    actual_brightness: usize,
    brightness: usize,
    max_brightness: usize
}

impl Backlight {
    fn from_name(file_name: &OsString) -> Result<Self, Error>  {
        let mut path_buf = PathBuf::from("/sys/class/backlight");
        path_buf.push(file_name);
        path_buf.push("brightness");
        let brightness = try!(read_file_to_usize(&path_buf));
        path_buf.pop();
        path_buf.push("max_brightness");
        let max_brightness = try!(read_file_to_usize(&path_buf));
        path_buf.pop();
        path_buf.push("actual_brightness");
        let actual_brightness = try!(read_file_to_usize(&path_buf));
        Ok(Self {
            name: file_name.clone(),
            actual_brightness: actual_brightness,
            brightness: brightness,
            max_brightness: max_brightness
        })
    }
    fn set_brightness(&mut self, new_value: usize) {
        let mut path_buf = PathBuf::from("/sys/class/backlight");
        path_buf.push(&self.name);
        path_buf.push("brightness");
        write_file(&path_buf, new_value.to_string().as_bytes()).expect("Couldn't write to brightness file");
        self.brightness = read_file_to_usize(&path_buf).unwrap();
    }
}

impl fmt::Display for Backlight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Backlight Name: {:?}\n    Brightness: {}\n    Actual Brightness: {}\n    Max Brightness: {}\n", self.name, self.brightness, self.actual_brightness, self.max_brightness)
    }
}

fn main() {
    let mut backlight_name: Option<String> = None;
    let mut desired_brightness: Option<usize> = None;
    let mut should_print_backlights = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Control display backlight brightness");
        ap.refer(&mut desired_brightness).add_argument("brightness", StoreOption, "brightness to set");
        ap.refer(&mut backlight_name).add_option(&["-d", "--device"], StoreOption, "backlight device name");
        ap.refer(&mut should_print_backlights).add_option(&["-p", "--print"], StoreTrue, "list all backlights");
        ap.parse_args_or_exit();
    }
    if should_print_backlights {
        let bls = Backlights::new();
        bls.print_backlights();
        exit(0);
    } else {
        let mut backlight = match backlight_name.map(|x| OsString::from(x)) {
            Some(v) => match Backlight::from_name(&v) {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to open backlight: {:?}", e);
                    let bls = Backlights::new();
                    bls.print_backlights();
                    exit(1);
                }
            },
            None => {
                let bls = Backlights::new();
                bls.default_backlight().expect("Failed to open default backlight")
            }
        };
        match desired_brightness {
            Some(v) => {
                backlight.set_brightness(v);
            },
            None => println!("{:#}", backlight)
        };
    }
}
