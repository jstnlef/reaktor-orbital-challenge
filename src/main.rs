extern crate nalgebra;
extern crate ncollide;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use nalgebra::{Vector3, Point3, Identity, Norm};
use ncollide::bounding_volume::BoundingSphere;
use ncollide::query::{Ray, RayCast};

// Earth's radius in km
pub const EARTH_RADIUS: f64 = 6371.0;


fn main() {
    let file_path = Path::new("data_file.txt");
    let (satellites, route) = parse_data_file(file_path);
    let network = generate_line_of_sight_network(&satellites, route);
    let path_of_signal = network.transmit_signal(route);
    println!("{:?}", satellites);
    println!("{:?}", route);
    println!("{}", path_of_signal.join(","));
}


fn convert_lat_long_to_vector(latitude: f64, longtitude: f64) -> Vector3<f64> {
    let lat = latitude.to_radians();
    let long = longtitude.to_radians();
    let x = EARTH_RADIUS * lat.cos() * long.cos();
    let y = EARTH_RADIUS * lat.cos() * long.sin();
    let z = EARTH_RADIUS * lat.sin();
    Vector3::new(x, y, z)
}

// 2 Vectors have line of sight if the vector created between the 2 doesn't intersect the earth.
pub fn has_line_of_sight(v1: Vector3<f64>, v2: Vector3<f64>) -> bool {
    let earth = BoundingSphere::new(Point3::new(0.0, 0.0, 0.0), EARTH_RADIUS);
    let direction = (v2 - v1).normalize();
    let ray = Ray::new(v1.to_point(), direction);
    !earth.intersects_ray(&Identity::new(), &ray)
}


fn generate_line_of_sight_network(satellites: &[Satellite], route: Route) -> Network {
    Network{}
}


#[derive(Debug)]
struct Network {

}
impl Network {
    fn transmit_signal(&self, route: Route) -> Vec<String> {
        vec!["test".to_string(), "test2".to_string()]
    }
}


#[derive(Clone, Copy, Debug)]
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


#[derive(Debug, PartialEq)]
pub struct Satellite {
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


mod tests {
    use super::*;
    use nalgebra::{Vector3};

    #[test]
    fn test_has_line_of_sight() {
        let a = Satellite::new("test".to_string(), 0.0, 0.0, 300.0);
        let b = Satellite::new("test2".to_string(), 0.0, 180.0, 300.0);
        assert_eq!(has_line_of_sight(a.position, b.position), false);
        let c = Satellite::new("test2".to_string(), 0.0, 0.0, 400.0);
        assert_eq!(has_line_of_sight(a.position, c.position), true);
    }

    #[test]
    fn test_satellite_creation_cartesian() {
        let on_earth = Satellite::new("testA".to_string(), 0.0, 0.0, 0.0);
        let in_space = Satellite::new("testB".to_string(), 0.0, 0.0, 200.0);

        assert_eq!(on_earth.position, Vector3::new(EARTH_RADIUS, 0.0, 0.0));
        assert_eq!(in_space.position, Vector3::new(EARTH_RADIUS + 200.0, 0.0, 0.0));
    }
}
