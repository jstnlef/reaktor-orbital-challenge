extern crate nalgebra;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use nalgebra::{Vector3, Norm};

// Earth's radius in km
const EARTH_RADIUS: f64 = 6371.0;


fn main() {
    let file_path = Path::new("data_file.txt");
    let (satellites, route) = parse_data_file(file_path);
    println!("{:?}", satellites);
    println!("{:?}", route);
}


fn convert_lat_long_to_vector(latitude: f64, longtitude: f64) -> Vector3<f64> {
    let lat = latitude.to_radians();
    let long = longtitude.to_radians();
    let x = EARTH_RADIUS * lat.cos() * long.cos();
    let y = EARTH_RADIUS * lat.cos() * long.sin();
    let z = EARTH_RADIUS * lat.sin();
    Vector3::new(x, y, z)
}


#[derive(Debug)]
struct Route {
    start: Vector3<f64>,
    end: Vector3<f64>,
}
impl Route {
    fn new(start_lat: f64, start_long: f64, end_lat: f64, end_long: f64) -> Self {
        let start = convert_lat_long_to_vector(start_lat, start_long);
        let end = convert_lat_long_to_vector(end_lat, end_long);
        Route {
            start: start,
            end: end
        }
    }
}


#[derive(Debug)]
struct Satellite {
    id: String,
    position: Vector3<f64>
}
impl Satellite {
    fn new(id: String, latitude: f64, longtitude: f64, altitude: f64) -> Self {
        let v = convert_lat_long_to_vector(latitude, longtitude);
        Satellite {
            id: id,
            position: v + (altitude * v.normalize())
        }
    }
}


fn parse_data_file(path: &Path) -> (Vec<Satellite>, Route) {
    let file_path = Path::new(path);
    let display = path.display();

    let file = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => BufReader::new(file)
    };

    let mut satellites = Vec::new();
    let mut route = None;

    for (i, line) in file.lines().enumerate() {
        let s = line.unwrap();
        if i == 0 {
            println!("Parsing file {}", s);
        } else if s.starts_with("ROUTE") {
            let parsed: Vec<f64> = s.split(",").skip(1).map(|s| s.parse().unwrap()).collect();
            route = Some(Route::new(parsed[0], parsed[1], parsed[2], parsed[3]));
        } else {
            let parsed: Vec<&str> = s.split(",").collect();
            satellites.push(Satellite::new(
                parsed[0].to_string(),
                parsed[1].parse::<f64>().unwrap(),
                parsed[2].parse::<f64>().unwrap(),
                parsed[3].parse::<f64>().unwrap()
            ));
        }
    }

    (satellites, route.unwrap())
}
