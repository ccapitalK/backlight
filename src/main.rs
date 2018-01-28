extern crate argparse;

use std::fs::{read_dir, File};
use std::ffi::OsString;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::process::exit;

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

static mut _AVAILABLE_BACKLIGHTS: Option<Vec<OsString>> = None;

fn initialize_ab() {
    unsafe {
        _AVAILABLE_BACKLIGHTS = Some(read_dir("/sys/class/backlight")
            .expect("Could not open /sys/class/backlights")
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap().file_name())
            .collect());
    }
}

fn available_backlights() -> Option<Vec<OsString>> {
    unsafe {
        _AVAILABLE_BACKLIGHTS.clone()
    }
}

#[derive(Debug)]
struct Backlight {
    name: OsString,
    brightness: usize,
    max_brightness: usize
}

impl Backlight {
    fn default() -> Result<Self, Error> {
        let ab = available_backlights().unwrap();
        let name = if ab.contains(&OsString::from("default")) {
            OsString::from("default")
        } else if ab.len() > 0 {
            ab[0].clone()
        } else {
            return Err(Error::new(ErrorKind::Other, "No backlights detected"))
        };
        Self::from_name(&name)
    }
    fn from_name(file_name: &OsString) -> Result<Self, Error>  {
        if available_backlights().unwrap().contains(file_name) {
            let mut path_buf = PathBuf::from("/sys/class/backlight");
            path_buf.push(file_name);
            path_buf.push("brightness");
            let brightness = try!(read_file_to_usize(&path_buf));
            path_buf.pop();
            path_buf.push("max_brightness");
            let max_brightness = try!(read_file_to_usize(&path_buf));
            Ok(Self {
                name: file_name.clone(),
                brightness: brightness,
                max_brightness: max_brightness
            })
        } else {
            Err(Error::new(ErrorKind::Other, "Backlight not found"))
        }
    }
    fn set_brightness(&mut self, new_value: usize) {
        let mut path_buf = PathBuf::from("/sys/class/backlight");
        path_buf.push(&self.name);
        path_buf.push("brightness");
        write_file(&path_buf, new_value.to_string().as_bytes()).expect("Couldn't write to brightness file");
        self.brightness = read_file_to_usize(&path_buf).unwrap();
    }
}

fn print_backlights(){
    println!("Printing device names:");
    for name in available_backlights().unwrap() {
        println!("{:?}", name);
    }
}

fn main() {
    initialize_ab();
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
        print_backlights();
        exit(0);
    } else {
        let mut backlight = match backlight_name.map(|x| OsString::from(x)) {
            Some(v) => match Backlight::from_name(&v) {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to open backlight: {:?}", e);
                    print_backlights();
                    exit(1);
                }
            },
            None => Backlight::default().expect("Failed to open default backlight")
        };
        match desired_brightness {
            Some(v) => {
                backlight.set_brightness(v);
            },
            None => println!("{:#?}", backlight)
        };
    }
}
